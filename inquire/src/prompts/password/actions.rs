use crate::{
    input::InputAction,
    ui::{InnerAction, Key, KeyModifiers},
};

use super::config::PasswordConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum PasswordPromptAction {
    ValueInput(InputAction),
    ToggleDisplayMode,
}

impl InnerAction<PasswordConfig> for PasswordPromptAction {
    fn from_key(key: Key, config: &PasswordConfig) -> Option<Self> {
        let action = match key {
            Key::Char('r', m) | Key::Char('R', m)
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
