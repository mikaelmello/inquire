use std::ffi::OsString;

use once_cell::sync::Lazy;

use crate::{
    config::get_configuration,
    error::InquireResult,
    new_prompts::variants::editor::{get_default_editor_command, EditorPrompt},
};

use super::common::CommonConfig;

static DEFAULT_EDITOR: Lazy<OsString> = Lazy::new(get_default_editor_command);

/// This prompt is meant for cases where you need the user to write some text that might not fit in a single line, such as long descriptions or commit messages.
///
/// This prompt is gated via the `editor` because it depends on the `tempfile` crate.
///
/// This prompt's behavior is to ask the user to either open the editor - by pressing the `e` key - or submit the current text - by pressing the `enter` key. The user can freely open and close the editor as they wish, until they either cancel or submit.
///
/// The editor opened is set by default to `nano` on Unix environments and `notepad` on Windows environments. Additionally, if there's an editor set in either the `EDITOR` or `VISUAL` environment variables, it is used instead.
///
/// If the user presses `esc` while the editor is not open, it will be interpreted as the user canceling (or skipping) the operation, in which case the prompt call will return `Err(InquireError::OperationCanceled)`.
///
/// If the user presses `enter` without ever modyfing the temporary file, it will be treated as an empty submission. If this is unwanted behavior, you can control the user input by using validators.
///
/// Finally, this prompt allows a great range of customizable options as all others:
///
/// - **Prompt message**: Main message when prompting the user for input, `"What is your name?"` in the example above.
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Editor command and its args**: If you want to override the selected editor, you can pass over the command and additional args.
/// - **File extension**: Custom extension for the temporary file, useful as a proxy for proper syntax highlighting for example.
/// - **Predefined text**: Pre-defined text to be written to the temporary file before the user is allowed to edit it.
/// - **Validators**: Custom validators to the user's input, displaying an error message if the input does not pass the requirements.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
///   - By default, a successfully submitted answer is displayed to the user simply as `<received>`.
#[derive(Clone)]
pub struct Editor<'a> {
    common: CommonConfig<'a, String, &'a str>,
    config: EditorConfig,
}

impl<'a> Editor<'a> {
    common_config_builder_methods!(String, &'a str);

    /// Creates a [DateSelect] with the provided message, along with default configuration values.
    pub fn new(message: &'a str) -> Self {
        Self {
            common: CommonConfig {
                message: message.into(),
                help_message: None,
                default: None,
                formatter: Box::new(&|_| String::from("<received>")),
                validators: vec![],
                render_config: get_configuration(),
            },
            config: EditorConfig::default(),
        }
    }

    /// Sets the command to open the editor.
    pub fn with_editor_command(mut self, editor_command: impl Into<OsString>) -> Self {
        self.config.editor_command = editor_command.into();
        self
    }

    /// Sets the args for the command to open the editor.
    pub fn with_args(mut self, args: Vec<OsString>) -> Self {
        self.config.editor_args = args;
        self
    }

    /// Sets the predefined text to be written into the temporary file.
    pub fn with_predefined_text(mut self, text: impl Into<String>) -> Self {
        self.common.default = Some(text.into());
        self
    }

    fn inner_impl(self) -> InquireResult<(CommonConfig<'a, String, &'a str>, EditorPrompt)> {
        let default_value = self.common.default.clone();
        let common = self.common;
        let inner_impl = EditorPrompt::new(self.config, default_value)?;

        Ok((common, inner_impl))
    }
}

#[derive(Clone)]
pub(crate) struct EditorConfig {
    pub editor_command: OsString,
    pub editor_args: Vec<OsString>,
    pub file_extension: OsString,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            editor_command: DEFAULT_EDITOR.clone(),
            editor_args: vec![],
            file_extension: ".txt".into(),
        }
    }
}

/// Set of actions for an EditorPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EditorPromptAction {
    /// Open the editor.
    OpenEditor,
}
