use crate::ui::{InnerAction, Key};

use super::config::EditorConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum EditorPromptAction {
    OpenEditor,
}

impl<'a> InnerAction<EditorConfig<'a>> for EditorPromptAction {
    fn from_key(key: Key, _config: &EditorConfig) -> Option<Self> {
        let action = match key {
            Key::Char('e', _) => Self::OpenEditor,
            _ => return None,
        };

        Some(action)
    }
}
