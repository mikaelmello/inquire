use crate::{
    ui::{Key, KeyModifiers},
    InnerAction, InputAction,
};

use super::config::PathSelectConfig;

/// Set of actions for a MultiSelectPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PathSelectPromptAction {
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
    /// Navigates deeper in file tree
    NavigateDeeper,
    //// Navigated higher in file tree
    NavigateHigher,
}

impl InnerAction<PathSelectConfig> for PathSelectPromptAction {
    fn from_key(key: Key, config: &PathSelectConfig) -> Option<Self> {
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
            Key::Right(KeyModifiers::SHIFT) => Self::SelectAll,
            Key::Right(KeyModifiers::NONE) => Self::NavigateDeeper,
            Key::Left(KeyModifiers::SHIFT) => Self::ClearSelections,
            Key::Left(KeyModifiers::NONE) => Self::NavigateHigher,
            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::FilterInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
