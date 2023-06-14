use crate::{ui::Key, InnerAction};

use super::config::EditorConfig;

/// Set of actions for an EditorPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EditorPromptAction {
    /// Open the editor.
    OpenEditor,
}

impl<'a> InnerAction<EditorConfig<'a>> for EditorPromptAction {
    fn from_key(key: Key, _config: &EditorConfig<'_>) -> Option<Self> {
        let action = match key {
            Key::Char('e', _) => Self::OpenEditor,
            _ => return None,
        };

        Some(action)
    }
}
