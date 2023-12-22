use crate::input::InputActionResult;

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

impl From<InputActionResult> for ActionResult {
    fn from(input_action_result: InputActionResult) -> Self {
        match input_action_result {
            InputActionResult::Clean => Self::Clean,
            InputActionResult::ContentChanged | InputActionResult::PositionChanged => {
                Self::NeedsRedraw
            }
        }
    }
}
