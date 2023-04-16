use crate::Editor;
use std::ffi::OsStr;

#[derive(Copy, Clone, Debug)]
pub struct EditorConfig<'a> {
    pub editor_command: &'a OsStr,
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
