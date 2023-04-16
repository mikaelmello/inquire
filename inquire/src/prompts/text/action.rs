use crate::{
    input::InputAction,
    ui::{InnerAction, Key, KeyModifiers},
};

use super::config::TextConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum TextPromptAction {
    ValueInput(InputAction),
    MoveToSuggestionAbove,
    MoveToSuggestionBelow,
    MoveToSuggestionPageUp,
    MoveToSuggestionPageDown,
    UseCurrentSuggestion,
}

impl InnerAction<TextConfig> for TextPromptAction {
    fn from_key(key: Key, _config: &TextConfig) -> Option<Self> {
        let action = match key {
            Key::Up(KeyModifiers::NONE) => Self::MoveToSuggestionAbove,
            Key::PageUp => Self::MoveToSuggestionPageUp,

            Key::Down(KeyModifiers::NONE) => Self::MoveToSuggestionBelow,
            Key::PageDown => Self::MoveToSuggestionPageDown,

            Key::Tab => Self::UseCurrentSuggestion,

            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::ValueInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
