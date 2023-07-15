use crate::{ui::Key, InnerAction};

use super::config::EditorConfig;

/// Set of actions for an EditorPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EditorPromptAction {
    /// Open the editor.
    OpenEditor,
}

impl InnerAction for EditorPromptAction {
    type Config = EditorConfig;

    fn from_key(key: Key, _config: &EditorConfig) -> Option<Self> {
        let action = match key {
            Key::Char('e', _) => Self::OpenEditor,
            _ => return None,
        };

        Some(action)
    }
}
