use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    io::{Read, Write},
    path::Path,
    process,
};

use lazy_static::lazy_static;
use tempfile::NamedTempFile;

use crate::{
    error::{InquireError, InquireResult},
    formatter::StringFormatter,
    terminal::get_default_terminal,
    ui::{Backend, EditorBackend, Key, RenderConfig},
    validator::{ErrorMessage, StringValidator, Validation},
};

lazy_static! {
    static ref DEFAULT_EDITOR: OsString = get_default_editor_command();
}

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
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Command to open the editor.
    pub editor_command: &'a OsStr,

    /// Args to pass to the editor.
    pub editor_command_args: &'a [&'a OsStr],

    /// Extension of the file opened in the text editor, useful for syntax highlighting.
    ///
    /// The dot prefix should be included in the string, e.g. ".rs".
    pub file_extension: &'a str,

    /// Predefined text to be present on the text file on the text editor.
    pub predefined_text: Option<&'a str>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: StringFormatter<'a>,

    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Vec<Box<dyn StringValidator>>,

    /// RenderConfig to apply to the rendered interface.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub render_config: RenderConfig,
}

impl<'a> Editor<'a> {
    /// Default formatter, set to [DEFAULT_STRING_FORMATTER](crate::formatter::DEFAULT_STRING_FORMATTER)
    pub const DEFAULT_FORMATTER: StringFormatter<'a> = &|_| String::from("<received>");

    /// Default validators added to the [Editor] prompt, none.
    pub const DEFAULT_VALIDATORS: Vec<Box<dyn StringValidator>> = vec![];

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> = None;

    /// Creates a [Editor] with the provided message and default options.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            editor_command: &DEFAULT_EDITOR,
            editor_command_args: &[],
            file_extension: ".txt",
            predefined_text: None,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            validators: Self::DEFAULT_VALIDATORS,
            formatter: Self::DEFAULT_FORMATTER,
            render_config: RenderConfig::default(),
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Sets the predefined text to be written into the temporary file.
    pub fn with_predefined_text(mut self, text: &'a str) -> Self {
        self.predefined_text = Some(text);
        self
    }

    /// Sets the file extension of the temporary file.
    pub fn with_file_extension(mut self, file_extension: &'a str) -> Self {
        self.file_extension = file_extension;
        self
    }

    /// Sets the command to open the editor.
    pub fn with_editor_command(mut self, editor_command: &'a OsStr) -> Self {
        self.editor_command = editor_command;
        self
    }

    /// Sets the args for the command to open the editor.
    pub fn with_args(mut self, args: &'a [&'a OsStr]) -> Self {
        self.editor_command_args = args;
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: StringFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Adds a validator to the collection of validators. You might want to use this feature
    /// in case you need to require certain features from the user's answer, such as
    /// defining a limit of characters.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validator<V>(mut self, validator: V) -> Self
    where
        V: StringValidator + 'static,
    {
        // Directly make space for at least 5 elements, so we won't to re-allocate too often when
        // calling this function repeatedly.
        if self.validators.capacity() == 0 {
            self.validators.reserve(5);
        }

        self.validators.push(Box::new(validator));
        self
    }

    /// Adds the validators to the collection of validators in the order they are given.
    /// You might want to use this feature in case you need to require certain features
    /// from the user's answer, such as defining a limit of characters.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validators(mut self, validators: &[Box<dyn StringValidator>]) -> Self {
        for validator in validators {
            #[allow(clippy::clone_double_ref)]
            self.validators.push(validator.clone());
        }
        self
    }

    /// Sets the provided color theme to this prompt.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub fn with_render_config(mut self, render_config: RenderConfig) -> Self {
        self.render_config = render_config;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    ///
    /// This method is intended for flows where the user skipping/cancelling
    /// the prompt - by pressing ESC - is considered normal behavior. In this case,
    /// it does not return `Err(InquireError::OperationCanceled)`, but `Ok(None)`.
    ///
    /// Meanwhile, if the user does submit an answer, the method wraps the return
    /// type with `Some`.
    pub fn prompt_skippable(self) -> InquireResult<Option<String>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<String> {
        let terminal = get_default_terminal()?;
        let mut backend = Backend::new(terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(crate) fn prompt_with_backend<B: EditorBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<String> {
        EditorPrompt::new(self)?.prompt(backend)
    }
}

struct EditorPrompt<'a> {
    message: &'a str,
    editor_command: &'a OsStr,
    editor_command_args: &'a [&'a OsStr],
    help_message: Option<&'a str>,
    formatter: StringFormatter<'a>,
    validators: Vec<Box<dyn StringValidator>>,
    error: Option<ErrorMessage>,
    tmp_file: NamedTempFile,
}

impl<'a> From<&'a str> for Editor<'a> {
    fn from(val: &'a str) -> Self {
        Editor::new(val)
    }
}

impl<'a> EditorPrompt<'a> {
    pub fn new(so: Editor<'a>) -> InquireResult<Self> {
        Ok(Self {
            message: so.message,
            editor_command: so.editor_command,
            editor_command_args: so.editor_command_args,
            help_message: so.help_message,
            formatter: so.formatter,
            validators: so.validators,
            error: None,
            tmp_file: Self::create_file(so.file_extension, so.predefined_text)?,
        })
    }

    fn create_file(
        file_extension: &str,
        predefined_text: Option<&str>,
    ) -> std::io::Result<NamedTempFile> {
        let mut tmp_file = tempfile::Builder::new()
            .prefix("tmp-")
            .suffix(file_extension)
            .rand_bytes(10)
            .tempfile()?;

        if let Some(predefined_text) = predefined_text {
            tmp_file.write_all(predefined_text.as_bytes())?;
            tmp_file.flush()?;
        }

        Ok(tmp_file)
    }

    fn run_editor(&mut self) -> InquireResult<()> {
        process::Command::new(self.editor_command)
            .args(self.editor_command_args)
            .arg(self.tmp_file.path())
            .spawn()?
            .wait()?;

        Ok(())
    }

    fn render<B: EditorBackend>(&mut self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        backend.frame_setup()?;

        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        let path = Path::new(self.editor_command);
        let editor_name = path
            .file_stem()
            .and_then(|f| f.to_str())
            .unwrap_or("editor");

        backend.render_prompt(prompt, editor_name)?;

        if let Some(message) = self.help_message {
            backend.render_help_message(message)?;
        }

        backend.frame_finish()?;

        Ok(())
    }

    fn validate_current_answer(&self) -> InquireResult<Validation> {
        let cur_answer = self.cur_answer()?;
        for validator in &self.validators {
            match validator.validate(&cur_answer) {
                Ok(Validation::Valid) => {}
                Ok(Validation::Invalid(msg)) => return Ok(Validation::Invalid(msg)),
                Err(err) => return Err(InquireError::Custom(err)),
            }
        }

        Ok(Validation::Valid)
    }

    fn cur_answer(&self) -> InquireResult<String> {
        let mut read_handler = fs::File::open(self.tmp_file.path())?;
        let mut submission = String::new();
        read_handler.read_to_string(&mut submission)?;

        let len = submission.trim_end_matches(&['\n', '\r'][..]).len();
        submission.truncate(len);

        Ok(submission)
    }

    fn prompt<B: EditorBackend>(mut self, backend: &mut B) -> InquireResult<String> {
        let final_answer = loop {
            self.render(backend)?;

            let key = backend.read_key()?;

            match key {
                Key::Interrupt => interrupt_prompt!(),
                Key::Cancel => cancel_prompt!(backend, self.message),
                Key::Char('e', _) => self.run_editor()?,
                Key::Submit => match self.validate_current_answer()? {
                    Validation::Valid => break self.cur_answer()?,
                    Validation::Invalid(msg) => self.error = Some(msg),
                },
                _ => {}
            }
        };

        let formatted = (self.formatter)(&final_answer);

        finish_prompt_with_answer!(backend, self.message, &formatted, final_answer);
    }
}

fn get_default_editor_command() -> OsString {
    let mut default_editor = if cfg!(windows) {
        String::from("notepad")
    } else {
        String::from("nano")
    };

    if let Ok(editor) = env::var("EDITOR") {
        if !editor.is_empty() {
            default_editor = editor;
        }
    }

    if let Ok(editor) = env::var("VISUAL") {
        if !editor.is_empty() {
            default_editor = editor;
        }
    }

    default_editor.into()
}
