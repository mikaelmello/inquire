use crate::{
    ui::{Key, KeyModifiers},
    InnerAction, InputAction,
};

use super::config::MultiSelectConfig;

/// Set of actions for a MultiSelectPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MultiSelectPromptAction {
    /// Action on the value text input handler.
    FilterInput(InputAction),
    /// Moves the cursor to the option above.
    MoveUp,
    /// Moves the cursor to the option below.
    MoveDown,
    /// Moves the cursor to the page above.
    PageUp,
    /// Moves the cursor to the page below.
    PageDown,
    /// Moves the cursor to the start of the list.
    MoveToStart,
    /// Moves the cursor to the end of the list.
    MoveToEnd,
    /// Toggles the selection of the current option.
    ToggleCurrentOption,
    /// Selects all options.
    SelectAll,
    /// Deselects all options.
    ClearSelections,
}

impl InnerAction for MultiSelectPromptAction {
    type Config = MultiSelectConfig;

    fn from_key(key: Key, config: &MultiSelectConfig) -> Option<Self> {
        if config.vim_mode {
            let action = match key {
                Key::Char('h', KeyModifiers::NONE) => Some(Self::ClearSelections),
                Key::Char('k', KeyModifiers::NONE) => Some(Self::MoveUp),
                Key::Char('j', KeyModifiers::NONE) => Some(Self::MoveDown),
                Key::Char('l', KeyModifiers::NONE) => Some(Self::SelectAll),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        let action = match key {
            Key::Up(KeyModifiers::NONE) | Key::Char('p', KeyModifiers::CONTROL) => Self::MoveUp,
            Key::PageUp(_) => Self::PageUp,
            Key::Home => Self::MoveToStart,

            Key::Down(KeyModifiers::NONE) | Key::Char('n', KeyModifiers::CONTROL) => Self::MoveDown,
            Key::PageDown(_) => Self::PageDown,
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
