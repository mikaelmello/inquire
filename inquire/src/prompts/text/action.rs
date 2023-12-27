use crate::{
    ui::{Key, KeyModifiers},
    InnerAction, InputAction,
};

use super::config::TextConfig;

/// Set of actions for a TextPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum TextPromptAction {
    /// Action on the value text input handler.
    ValueInput(InputAction),
    /// When a suggestion list exists, moves the cursor to the option above.
    MoveToSuggestionAbove,
    /// When a suggestion list exists, moves the cursor to the option below.
    MoveToSuggestionBelow,
    /// When a suggestion list exists, moves the cursor to the page above.
    MoveToSuggestionPageUp,
    /// When a suggestion list exists, moves the cursor to the page below.
    MoveToSuggestionPageDown,
    /// When a suggestion list exists, autocompletes the text input with the current suggestion.
    UseCurrentSuggestion,
}

impl InnerAction for TextPromptAction {
    type Config = TextConfig;

    fn from_key(key: Key, _config: &TextConfig) -> Option<Self> {
        let action = match key {
            Key::Up(KeyModifiers::NONE) | Key::Char('p', KeyModifiers::CONTROL) => {
                Self::MoveToSuggestionAbove
            }
            Key::PageUp(_) => Self::MoveToSuggestionPageUp,

            Key::Down(KeyModifiers::NONE) | Key::Char('n', KeyModifiers::CONTROL) => {
                Self::MoveToSuggestionBelow
            }
            Key::PageDown(_) => Self::MoveToSuggestionPageDown,

            Key::Tab => Self::UseCurrentSuggestion,

            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::ValueInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
