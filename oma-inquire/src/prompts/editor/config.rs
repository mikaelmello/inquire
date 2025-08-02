use crate::Editor;
use std::ffi::OsString;

/// Configuration settings used in the execution of an EditorPrompt.
#[derive(Clone, Debug)]
pub struct EditorConfig {
    /// The command to use to open the editor.
    pub editor_command: OsString,
    /// The arguments to pass to the editor command.
    pub editor_command_args: Vec<OsString>,
}

impl<'a> From<&Editor<'a>> for EditorConfig {
    fn from(value: &Editor<'a>) -> Self {
        Self {
            editor_command: value.editor_command.into(),
            editor_command_args: value.editor_command_args.iter().map(Into::into).collect(),
        }
    }
}
