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
                Key::Char('k', KeyModifiers::NONE) => Some(MultiSelectPromptAction::MoveUp),
                Key::Char('j', KeyModifiers::NONE) => Some(MultiSelectPromptAction::MoveDown),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        let action = match key {
            Key::Up(KeyModifiers::NONE) => MultiSelectPromptAction::MoveUp,
            Key::PageUp => MultiSelectPromptAction::PageUp,
            Key::Home => MultiSelectPromptAction::MoveToStart,

            Key::Down(KeyModifiers::NONE) => MultiSelectPromptAction::MoveDown,
            Key::PageDown => MultiSelectPromptAction::PageDown,
            Key::End => MultiSelectPromptAction::MoveToEnd,

            Key::Char(' ', KeyModifiers::NONE) => MultiSelectPromptAction::ToggleCurrentOption,
            Key::Right(KeyModifiers::NONE) => MultiSelectPromptAction::SelectAll,
            Key::Left(KeyModifiers::NONE) => MultiSelectPromptAction::ClearSelections,
            key => match InputAction::from_key(key, &()) {
                Some(action) => MultiSelectPromptAction::FilterInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
