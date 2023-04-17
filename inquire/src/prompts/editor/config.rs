use crate::Editor;
use std::ffi::OsStr;

/// Configuration settings used in the execution of an EditorPrompt.
#[derive(Copy, Clone, Debug)]
pub struct EditorConfig<'a> {
    /// The command to use to open the editor.
    pub editor_command: &'a OsStr,
    /// The arguments to pass to the editor command.
    pub editor_command_args: &'a [&'a OsStr],
}

impl<'a> From<&Editor<'a>> for EditorConfig<'a> {
    fn from(value: &Editor<'a>) -> Self {
        Self {
            editor_command: value.editor_command,
            editor_command_args: value.editor_command_args,
        }
    }
}
