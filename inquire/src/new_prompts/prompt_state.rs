use super::action_result::ActionResult;

pub enum PromptState {
    Active(ActionResult),
    Canceled,
    Submitted,
}

impl PromptState {
    pub fn needs_rendering(&self) -> bool {
        match self {
            Self::Active(result) => *result == ActionResult::NeedsRedraw,
            Self::Canceled | Self::Submitted => true,
        }
    }

    pub fn require_redraw(&mut self) {
        match self {
            Self::Active(result) => *result = ActionResult::NeedsRedraw,
            Self::Canceled | Self::Submitted => {}
        }
    }
}
