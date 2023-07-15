use crate::{
    ui::{Key, KeyModifiers},
    InnerAction, InputAction,
};

use super::config::PasswordConfig;

/// Set of actions for a PasswordPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PasswordPromptAction {
    /// Action on the value text input handler.
    ValueInput(InputAction),
    /// Toggles the display mode between plain text and the initial one.
    ToggleDisplayMode,
}

impl InnerAction for PasswordPromptAction {
    type Config = PasswordConfig;

    fn from_key(key: Key, config: &PasswordConfig) -> Option<Self> {
        let action = match key {
            Key::Char('r' | 'R', m)
                if m.contains(KeyModifiers::CONTROL) && config.enable_display_toggle =>
            {
                Self::ToggleDisplayMode
            }
            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::ValueInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
