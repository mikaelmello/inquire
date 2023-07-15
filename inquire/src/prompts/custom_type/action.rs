use crate::{ui::Key, InnerAction, InputAction};

use super::config::CustomTypeConfig;

/// Set of actions for a CustomTypePrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CustomTypePromptAction {
    /// Action on the value text input handler.
    ValueInput(InputAction),
}

impl InnerAction for CustomTypePromptAction {
    type Config = CustomTypeConfig;

    fn from_key(key: Key, _config: &CustomTypeConfig) -> Option<Self> {
        let action = match InputAction::from_key(key, &()) {
            Some(action) => Self::ValueInput(action),
            None => return None,
        };

        Some(action)
    }
}
