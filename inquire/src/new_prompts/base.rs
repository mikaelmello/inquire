use crate::{
    error::InquireResult,
    formatter::SubmissionFormatter,
    input::Input,
    ui::CommonBackend,
    validator::{ErrorMessage, SubmissionValidator, Validation},
    InnerAction, InputAction, InquireError,
};

use super::{
    action::{ControlAction, ParseKey},
    action_result::ActionResult,
    prompt_state::PromptState,
};

pub trait PromptImpl<B> {
    type Action: Copy;
    type Output;

    fn setup(&mut self) -> InquireResult<()> {
        Ok(())
    }

    fn pre_cancel(&mut self) -> InquireResult<bool> {
        Ok(true)
    }

    fn has_inline_text_input() -> bool {
        false
    }

    fn handle(&mut self, action: Self::Action) -> InquireResult<ActionResult>;
    fn render(&self, message: &str, backend: &mut B) -> InquireResult<()>;
    fn current_submission(&self) -> &Self::Output;
    fn into_output(self) -> Self::Output;
}

pub struct Prompt<'a, InnerImpl, InnerActionType, Backend>
where
    InnerImpl: PromptImpl<Backend, Action = InnerActionType>,
    Backend: CommonBackend,
{
    message: String,
    help_message: Option<String>,
    validators: Vec<Box<dyn SubmissionValidator<InnerImpl::Output>>>,
    formatter: Box<dyn SubmissionFormatter<InnerImpl::Output>>,
    backend: &'a mut Backend,
    inner_impl: InnerImpl,
    error_message: Option<ErrorMessage>,
    state: PromptState,
    input: Option<Input>,
}

impl<'a, InnerImpl, InnerActionType, Backend> Prompt<'a, InnerImpl, InnerActionType, Backend>
where
    InnerImpl: PromptImpl<Backend, Action = InnerActionType>,
    Backend: CommonBackend,
    InnerActionType: ParseKey,
{
    pub fn new(
        message: impl Into<String>,
        help_message: Option<impl Into<String>>,
        validators: Vec<Box<dyn SubmissionValidator<InnerImpl::Output>>>,
        formatter: Box<dyn SubmissionFormatter<InnerImpl::Output>>,
        backend: &'a mut Backend,
        inner_impl: InnerImpl,
    ) -> Self {
        Self {
            message: message.into(),
            help_message: help_message.map(|s| s.into()),
            validators,
            formatter,
            backend,
            inner_impl,
            error_message: None,
            state: PromptState::Active(ActionResult::NeedsRedraw),
            input: match InnerImpl::has_inline_text_input() {
                true => Some(Input::new()),
                false => None,
            },
        }
    }

    fn render(&mut self) -> InquireResult<()> {
        if !self.state.needs_rendering() {
            return Ok(());
        }

        self.backend.frame_setup()?;

        match self.state {
            PromptState::Canceled => self.backend.render_canceled_prompt(&self.message)?,
            PromptState::Submitted => self.backend.render_prompt_with_answer(
                &self.message,
                &self.formatter.format(self.inner_impl.current_submission()),
            )?,
            PromptState::Active(_) => {
                if let Some(error_message) = self.error_message.as_ref() {
                    self.backend.render_error_message(error_message)?;
                }

                self.inner_impl.render(&self.message, self.backend)?;

                if let Some(help_message) = self.help_message.as_ref() {
                    self.backend.render_help_message(help_message)?;
                }
            }
        }

        self.backend.frame_finish()?;
        Ok(())
    }

    fn submit(&mut self) -> InquireResult<()> {
        let cur_submission = self.inner_impl.current_submission();

        for validator in &self.validators {
            match validator.validate(cur_submission) {
                Ok(Validation::Valid) => {}
                Ok(Validation::Invalid(msg)) => {
                    self.error_message = Some(msg);
                    self.state.require_redraw();
                    return Ok(());
                }
                Err(err) => return Err(InquireError::Custom(err)),
            }
        }

        self.state = PromptState::Submitted;

        Ok(())
    }

    fn cancel(&mut self) -> InquireResult<()> {
        let pre_cancel_result = self.inner_impl.pre_cancel()?;

        if pre_cancel_result {
            self.state = PromptState::Canceled;
        }

        Ok(())
    }

    pub fn prompt(mut self) -> InquireResult<InnerImpl::Output> {
        self.inner_impl.setup()?;

        let mut last_handle = ActionResult::NeedsRedraw;

        loop {
            self.render()?;
            last_handle.reset();

            let key = self.backend.read_key()?;

            if let Some(control_action) = ControlAction::from_key(key) {
                match control_action {
                    ControlAction::Submit => self.submit()?,
                    ControlAction::Cancel => self.cancel()?,
                    ControlAction::Interrupt => return Err(InquireError::OperationInterrupted),
                };
            };

            if let Some(prompt_action) = InnerActionType::from_key(key) {
                let result = self.inner_impl.handle(prompt_action)?;
                last_handle.merge(result);
            }

            if let Some(input) = self.input.as_mut() {
                if let Some(input_action) = InputAction::from_key(key, &()) {
                    let result = input.handle(input_action);
                    last_handle.merge(result.into());
                }
            }

            match self.state {
                PromptState::Canceled => return Err(InquireError::OperationCanceled),
                PromptState::Submitted => break,
                PromptState::Active(_) => {}
            }
        }

        self.render()?;

        Ok(self.inner_impl.into_output())
    }
}
