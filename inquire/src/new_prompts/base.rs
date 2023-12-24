use std::{
    borrow::{Borrow, BorrowMut},
    ops::Deref,
};

use crate::{
    error::InquireResult,
    formatter::SubmissionFormatter,
    input::Input,
    ui::CommonBackend,
    validator::{ErrorMessage, SubmissionValidator, Validation},
    InquireError,
};

use super::{
    action::{Action, ControlAction, ParseKey},
    action_result::ActionResult,
    prompt_state::PromptState,
};

pub trait PromptImpl<'a, B> {
    type Action: Copy;
    type Output;
    type OutputAsArgument;

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
    fn current_submission(&self) -> Self::OutputAsArgument;
    fn into_output(self) -> Self::Output;
}

pub struct Prompt<'a, InnerImpl, InnerActionType, Backend>
where
    InnerImpl: PromptImpl<'a, Backend, Action = InnerActionType>,
    Backend: CommonBackend,
{
    message: String,
    help_message: Option<String>,
    validators: Vec<Box<dyn SubmissionValidator<InnerImpl::OutputAsArgument>>>,
    formatter: Box<dyn SubmissionFormatter<InnerImpl::OutputAsArgument>>,
    backend: &'a mut Backend,
    inner_impl: InnerImpl,
    error_message: Option<ErrorMessage>,
    state: PromptState,
    input: Option<Input>,
}

impl<'a, InnerImpl, InnerActionType, Backend> Prompt<'a, InnerImpl, InnerActionType, Backend>
where
    InnerImpl: PromptImpl<'a, Backend, Action = InnerActionType>,
    Backend: CommonBackend,
    InnerActionType: ParseKey,
{
    pub fn new(
        message: impl Into<String>,
        help_message: Option<impl Into<String>>,
        validators: Vec<Box<dyn SubmissionValidator<InnerImpl::OutputAsArgument>>>,
        formatter: Box<dyn SubmissionFormatter<InnerImpl::OutputAsArgument>>,
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

    fn submit(&mut self) -> InquireResult<PromptState> {
        let cur_submission = self.inner_impl.current_submission();

        for validator in &self.validators {
            match validator.validate(cur_submission) {
                Ok(Validation::Valid) => {}
                Ok(Validation::Invalid(msg)) => {
                    self.error_message = Some(msg);
                    return Ok(PromptState::Active(ActionResult::NeedsRedraw));
                }
                Err(err) => return Err(InquireError::Custom(err)),
            }
        }

        Ok(PromptState::Submitted)
    }

    fn cancel(&mut self) -> InquireResult<PromptState> {
        let pre_cancel_result = self.inner_impl.pre_cancel()?;

        if pre_cancel_result {
            return Ok(PromptState::Canceled);
        }

        Ok(self.state)
    }

    pub fn prompt(mut self) -> InquireResult<InnerImpl::Output> {
        self.inner_impl.setup()?;

        loop {
            self.render()?;

            let key = self.backend.read_key()?;

            self.state = match Action::<InnerImpl::Action>::from_key(key) {
                Some(action) => match action {
                    Action::Control(control_action) => match control_action {
                        ControlAction::Submit => self.submit()?,
                        ControlAction::Cancel => self.cancel()?,
                        ControlAction::Interrupt => return Err(InquireError::OperationInterrupted),
                    },
                    Action::Inner(inner_action) => {
                        let result = self.inner_impl.handle(inner_action)?;
                        PromptState::Active(result)
                    }
                    Action::Input(input_action) => match self.input.as_mut() {
                        Some(input) => {
                            let result = input.handle(input_action);
                            PromptState::Active(result.into())
                        }
                        None => self.state,
                    },
                },
                None => self.state,
            };

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
