//! Definitions of common behavior shared amongst all different prompt types.

use crate::{error::InquireResult, input::InputActionResult, ui::CommonBackend, InquireError};

use super::action::{Action, InnerAction};

/// Represents the result of an action on the prompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionResult {
    /// The action resulted in a state change that requires the prompt to be
    /// re-rendered.
    NeedsRedraw,

    /// The action either didn't result in a state change or the state
    /// change does not require a redraw.
    Clean,
}

impl ActionResult {
    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::NeedsRedraw, _) | (_, Self::NeedsRedraw) => Self::NeedsRedraw,
            (Self::Clean, Self::Clean) => Self::Clean,
        }
    }

    /// Returns whether the action requires a redraw.
    pub fn needs_redraw(&self) -> bool {
        matches!(self, Self::NeedsRedraw)
    }
}

impl From<InputActionResult> for ActionResult {
    fn from(value: InputActionResult) -> Self {
        if value.needs_redraw() {
            Self::NeedsRedraw
        } else {
            Self::Clean
        }
    }
}

/// Shared behavior among all different prompt types.
pub trait Prompt<Backend>
where
    Backend: CommonBackend,
    Self: Sized,
{
    type Config;
    type InnerAction: InnerAction<Config = Self::Config>;
    type Output;

    /// Prompt header rendered to the user.
    fn message(&self) -> &str;

    /// Returns the underlying settings of the prompt, used, among other
    /// goals, to parse a key event into a prompt action.
    ///
    /// For example, a prompt might be configured to have vim mode enabled
    /// or disabled, which affects how certain key events are parsed into
    /// actions to the prompt.
    fn config(&self) -> &Self::Config;

    /// Hook called when a prompt is finished. Returns a string
    /// to be rendered to the user as the final submission to the prompt.
    ///
    /// # Arguments
    ///
    /// * `answer` - Answer returned by the prompt.
    fn format_answer(&self, answer: &Self::Output) -> String;

    /// Hook called when a prompt is first started, before the first
    /// draw happens.
    fn setup(&mut self) -> InquireResult<()> {
        Ok(())
    }

    /// Hook called when an input to cancel the prompt is triggered.
    ///
    /// Returns whether the prompt can be terminated.
    fn pre_cancel(&mut self) -> InquireResult<bool> {
        Ok(true)
    }

    /// Hook called when the user submits the answer to the prompt.
    ///
    /// On success, it should return `Some(ReturnType)` when the user
    /// submission is valid and the prompt can graciously return.
    ///
    /// If the user submission is invalid or should be rejected for some reason,
    /// this method should return `Ok(None)`.
    ///
    /// On `Err(*)`, the prompt is teared down.
    fn submit(&mut self) -> InquireResult<Option<Self::Output>>;

    /// Entrypoint for any business logic for the prompt. Returns the result
    /// of the action. If the result is `Clean`, the prompt will
    /// not be re-rendered.
    ///
    /// On the usual path, users' key presses are parsed into prompt actions,
    /// which are then submitted to this method to be handled.
    ///
    /// On testing scenarios, developers might provide a stream of actions
    /// to the prompt, which will then be submitted to this method just the same.
    fn handle(&mut self, action: Self::InnerAction) -> InquireResult<ActionResult>;

    /// Hook called for the rendering of the prompt UI.
    ///
    /// The implementation should **not** call neither `frame_setup` or
    /// `frame_finish` methods of the underlying backend, as this is handled
    /// by the top-level prompt method.
    fn render(&self, backend: &mut Backend) -> InquireResult<()>;

    /// Top-level implementation of a prompt's flow.
    ///
    /// This should not be reimplemented by types that implement this trait,
    /// unless the situation really warrants it.
    fn prompt(mut self, backend: &mut Backend) -> InquireResult<Self::Output> {
        self.setup()?;

        let mut last_handle = ActionResult::NeedsRedraw;
        let final_answer = loop {
            if last_handle.needs_redraw() {
                backend.frame_setup()?;
                self.render(backend)?;
                backend.frame_finish()?;
                last_handle = ActionResult::Clean;
            }

            let key = backend.read_key()?;
            let action = Action::from_key(key, self.config());

            if let Some(action) = action {
                last_handle = match action {
                    Action::Submit => {
                        if let Some(answer) = self.submit()? {
                            break answer;
                        }
                        ActionResult::NeedsRedraw
                    }
                    Action::Cancel => {
                        let pre_cancel_result = self.pre_cancel()?;

                        if pre_cancel_result {
                            backend.frame_setup()?;
                            backend.render_canceled_prompt(self.message())?;
                            backend.frame_finish()?;
                            return Err(InquireError::OperationCanceled);
                        }

                        ActionResult::NeedsRedraw
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
