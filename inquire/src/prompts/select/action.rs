use crate::{
    input::InputAction,
    ui::{InnerAction, Key, KeyModifiers},
};

use super::config::SelectConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum SelectPromptAction {
    FilterInput(InputAction),
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    MoveToStart,
    MoveToEnd,
}

impl InnerAction<SelectConfig> for SelectPromptAction {
    fn from_key(key: Key, config: &SelectConfig) -> Option<Self> {
        if config.vim_mode {
            let action = match key {
                Key::Char('k', KeyModifiers::NONE) => Some(Self::MoveUp),
                Key::Char('j', KeyModifiers::NONE) => Some(Self::MoveDown),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        let action = match key {
            Key::Up(KeyModifiers::NONE) => Self::MoveUp,
            Key::PageUp => Self::PageUp,
            Key::Home => Self::MoveToStart,

            Key::Down(KeyModifiers::NONE) => Self::MoveDown,
            Key::PageDown => Self::PageDown,
            Key::End => Self::MoveToEnd,

            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::FilterInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
