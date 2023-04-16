use crate::{
    input::InputAction,
    ui::{InnerAction, Key, KeyModifiers},
};

use super::config::MultiSelectConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum MultiSelectPromptAction {
    FilterInput(InputAction),
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    MoveToStart,
    MoveToEnd,
    ToggleCurrentOption,
    SelectAll,
    ClearSelections,
}

impl InnerAction<MultiSelectConfig> for MultiSelectPromptAction {
    fn from_key(key: Key, config: &MultiSelectConfig) -> Option<Self> {
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

            Key::Char(' ', KeyModifiers::NONE) => Self::ToggleCurrentOption,
            Key::Right(KeyModifiers::NONE) => Self::SelectAll,
            Key::Left(KeyModifiers::NONE) => Self::ClearSelections,
            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::FilterInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
