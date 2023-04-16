use crate::{
    error::InquireResult,
    ui::{Action, CommonBackend, InnerAction},
    InquireError,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HandleResult {
    Dirty,
    Clean,
}

impl HandleResult {
    pub fn join(self, other: HandleResult) -> HandleResult {
        match (self, other) {
            (HandleResult::Clean, HandleResult::Clean) => HandleResult::Clean,
            (_, _) => HandleResult::Dirty,
        }
    }

    pub fn from_bool_cmp(val: bool) -> HandleResult {
        if val {
            HandleResult::Dirty
        } else {
            HandleResult::Clean
        }
    }
}

pub trait Prompt<Backend, Config, IAction, ReturnType>
where
    Backend: CommonBackend,
    IAction: InnerAction<Config>,
    Self: Sized,
{
    fn message(&self) -> &str;
    fn config(&self) -> &Config;
    fn format_answer(&self, answer: &ReturnType) -> String;

    fn setup(&mut self) -> InquireResult<()> {
        Ok(())
    }

    /// Hook called when an input to cancel the prompt is triggered.
    ///
    /// Returns whether the prompt can be terminated.
    fn pre_cancel(&mut self) -> InquireResult<bool> {
        Ok(true)
    }

    fn submit(&mut self) -> InquireResult<Option<ReturnType>>;
    fn handle(&mut self, action: IAction) -> InquireResult<HandleResult>;
    fn render(&self, backend: &mut Backend) -> InquireResult<()>;

    fn prompt(mut self, backend: &mut Backend) -> InquireResult<ReturnType> {
        self.setup()?;

        let mut last_handle = HandleResult::Dirty;
        let final_answer = loop {
            if let HandleResult::Dirty = last_handle {
                backend.frame_setup()?;
                self.render(backend)?;
                backend.frame_finish()?;
                last_handle = HandleResult::Clean;
            }

            let key = backend.read_key()?;
            let action = Action::from_key(key, self.config());

            if let Some(action) = action {
                last_handle = match action {
                    Action::Submit => {
                        if let Some(answer) = self.submit()? {
                            break answer;
                        }
                        HandleResult::Clean
                    }
                    Action::Cancel => {
                        let pre_cancel_result = self.pre_cancel()?;

                        if pre_cancel_result {
                            backend.frame_setup()?;
                            backend.render_canceled_prompt(self.message())?;
                            backend.frame_finish()?;
                            return Err(InquireError::OperationCanceled);
                        }

                        HandleResult::Dirty
                    }
                    Action::Interrupt => return Err(InquireError::OperationInterrupted),
                    Action::Inner(inner_action) => self.handle(inner_action)?,
                };
            }
        };

        let formatted = self.format_answer(&final_answer);

        backend.frame_setup()?;
        backend.render_prompt_with_answer(self.message(), &formatted)?;
        backend.frame_finish()?;

        Ok(final_answer)
    }
}
