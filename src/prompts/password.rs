use crate::{
    config::get_configuration,
    error::{InquireError, InquireResult},
    formatter::StringFormatter,
    input::Input,
    terminal::get_default_terminal,
    ui::{Backend, Key, KeyModifiers, PasswordBackend, RenderConfig},
    validator::StringValidator,
};

/// Display modes of the text input of a password prompt.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PasswordDisplayMode {
    /// Password text input is not rendered at all, no indication of input.
    Hidden,

    /// Characters of the password text input are rendered marked as different
    /// characters, such as asterisks. These characters are configured in the
    /// render config.
    Masked,

    /// Password text input is fully rendered as a normal input, just like
    /// [Text](crate::Text) prompts.
    Full,
}

/// Prompt meant for secretive text inputs.
///
/// By default, the password prompt behaves like a standard one you'd see in common CLI applications: the user has no UI indicators about the state of the current input. They do not know how many characters they typed, or which character they typed, with no option to display the current text input.
///
/// However, you can still customize these and other behaviors if you wish:
/// - **Standard display mode**: Set the display mode of the text input among hidden, masked and full via the `PasswordDisplayMode` enum.
///   - Hidden: default behavior, no UI indicators.
///   - Masked: behaves like a normal text input, except that all characters of the input are masked to a special character, which is `'*'` by default but can be customized via `RenderConfig`.
///   - Full: behaves like a normal text input, no modifications.
/// - **Toggle display mode**: By enabling this feature by calling the `with_display_toggle_enabled()`, you allow the user to toggle between the standard display mode set and the full display mode.
///   - If you have set the standard display mode to hidden (which is also the default) or masked, the user can press `Ctrl+R` to change the display mode to `Full`, and `Ctrl+R` again to change it back to the standard one.
///   - Obviously, if you have set the standard display mode to `Full`, pressing `Ctrl+R` won't cause any changes.
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
///   - By default, it prints eight asterisk characters: `********`.
/// - **Validators**: Custom validators to make sure a given submitted input pass the specified requirements, e.g. not allowing empty inputs or requiring special characters.
///   - No validators are on by default.
///
/// Remember that for CLI applications it is standard to not allow use any display modes other than `Hidden` and to not allow the user to see the text input in any way. _Use the customization options at your discretion_.
///
/// # Example
///
/// ```no_run
///  use inquire::{validator::StringValidator, Password, PasswordDisplayMode};
///
///  let validator: StringValidator = &|input| if input.chars().count() < 10 {
///      Err(String::from("Keys must have at least 10 characters."))
///  } else {
///      Ok(())
///  };
///
///  let name = Password::new("Encryption Key:")
///      .with_display_toggle_enabled()
///      .with_display_mode(PasswordDisplayMode::Hidden)
///      .with_validator(validator)
///      .with_formatter(&|_| String::from("Input received"))
///      .with_help_message("It is recommended to generate a new one only for this purpose")
///      .prompt();
///
///  match name {
///      Ok(_) => println!("This doesn't look like a key."),
///      Err(_) => println!("An error happened when asking for your key, try again later."),
///  }
/// ```
#[derive(Clone)]
pub struct Password<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: StringFormatter<'a>,

    /// How the password input is displayed to the user.
    pub display_mode: PasswordDisplayMode,

    /// Whether to allow the user to toggle the display of the current password input by pressing the Ctrl+R hotkey.
    pub enable_display_toggle: bool,

    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Vec<StringValidator<'a>>,

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

impl<'a> Password<'a> {
    /// Default formatter, set to always display `"********"` regardless of input length.
    pub const DEFAULT_FORMATTER: StringFormatter<'a> = &|_| String::from("********");

    /// Default validators added to the [Password] prompt, none.
    pub const DEFAULT_VALIDATORS: Vec<StringValidator<'a>> = vec![];

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> = None;

    /// Default value for the allow display toggle variable.
    pub const DEFAULT_ENABLE_DISPLAY_TOGGLE: bool = false;

    /// Default password display mode.
    pub const DEFAULT_DISPLAY_MODE: PasswordDisplayMode = PasswordDisplayMode::Hidden;

    /// Creates a [Password] with the provided message and default options.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            enable_display_toggle: Self::DEFAULT_ENABLE_DISPLAY_TOGGLE,
            display_mode: Self::DEFAULT_DISPLAY_MODE,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            formatter: Self::DEFAULT_FORMATTER,
            validators: Self::DEFAULT_VALIDATORS,
            render_config: get_configuration(),
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Sets the flag to enable display toggling.
    pub fn with_display_toggle_enabled(mut self) -> Self {
        self.enable_display_toggle = true;
        self
    }

    /// Sets the standard display mode for the prompt.
    pub fn with_display_mode(mut self, mode: PasswordDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: StringFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Adds a validator to the collection of validators. You might want to use this feature
    /// in case you need to limit the user to specific choices, such as requiring
    /// special characters in the password.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validator(mut self, validator: StringValidator<'a>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Adds the validators to the collection of validators in the order they are given.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validators(mut self, validators: &[StringValidator<'a>]) -> Self {
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

    pub(in crate) fn prompt_with_backend<B: PasswordBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<String> {
        PasswordPrompt::from(self).prompt(backend)
    }
}

struct PasswordPrompt<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    input: Input,
    standard_display_mode: PasswordDisplayMode,
    display_mode: PasswordDisplayMode,
    enable_display_toggle: bool,
    formatter: StringFormatter<'a>,
    validators: Vec<StringValidator<'a>>,
    error: Option<String>,
}

impl<'a> From<Password<'a>> for PasswordPrompt<'a> {
    fn from(so: Password<'a>) -> Self {
        Self {
            message: so.message,
            help_message: so.help_message,
            standard_display_mode: so.display_mode,
            display_mode: so.display_mode,
            enable_display_toggle: so.enable_display_toggle,
            formatter: so.formatter,
            validators: so.validators,
            input: Input::new(),
            error: None,
        }
    }
}

impl<'a> From<&'a str> for Password<'a> {
    fn from(val: &'a str) -> Self {
        Password::new(val)
    }
}

impl<'a> PasswordPrompt<'a> {
    fn on_change(&mut self, key: Key) {
        match key {
            Key::Char('r', m) | Key::Char('R', m)
                if m.contains(KeyModifiers::CONTROL) && self.enable_display_toggle =>
            {
                self.toggle_display_mode();
            }
            _ => {
                self.input.handle_key(key);
            }
        };
    }

    fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            PasswordDisplayMode::Hidden => PasswordDisplayMode::Full,
            PasswordDisplayMode::Masked => PasswordDisplayMode::Full,
            PasswordDisplayMode::Full => self.standard_display_mode,
        }
    }

    fn get_final_answer(&self) -> Result<String, String> {
        for validator in &self.validators {
            match validator(self.input.content()) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(self.input.content().into())
    }

    fn render<B: PasswordBackend>(&mut self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        backend.frame_setup()?;

        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        match self.display_mode {
            PasswordDisplayMode::Hidden => {
                backend.render_prompt(prompt)?;
            }
            PasswordDisplayMode::Masked => {
                backend.render_prompt_with_masked_input(prompt, &self.input)?;
            }
            PasswordDisplayMode::Full => {
                backend.render_prompt_with_full_input(prompt, &self.input)?;
            }
        };

        if let Some(message) = self.help_message {
            backend.render_help_message(message)?;
        }

        backend.frame_finish()?;

        Ok(())
    }

    fn prompt<B: PasswordBackend>(mut self, backend: &mut B) -> InquireResult<String> {
        let final_answer: String;

        loop {
            self.render(backend)?;

            let key = backend.read_key()?;

            match key {
                Key::Interrupt => interrupt_prompt!(),
                Key::Cancel => cancel_prompt!(backend, self.message),
                Key::Submit => match self.get_final_answer() {
                    Ok(answer) => {
                        final_answer = answer;
                        break;
                    }
                    Err(err) => self.error = Some(err),
                },
                key => self.on_change(key),
            }
        }

        let formatted = (self.formatter)(&final_answer);

        finish_prompt_with_answer!(backend, self.message, &formatted, final_answer);
    }
}

#[cfg(test)]
#[cfg(feature = "crossterm")]
mod test {
    use super::Password;
    use crate::{
        terminal::crossterm::CrosstermTerminal,
        ui::{Backend, RenderConfig},
    };
    use crossterm::event::{KeyCode, KeyEvent};

    fn default<'a>() -> Password<'a> {
        Password::new("Question?")
    }

    macro_rules! text_to_events {
        ($text:expr) => {{
            $text.chars().map(KeyCode::Char)
        }};
    }

    macro_rules! password_test {
        ($name:ident,$input:expr,$output:expr) => {
            password_test! {$name, $input, $output, default()}
        };

        ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
            #[test]
            fn $name() {
                let read: Vec<KeyEvent> = $input.into_iter().map(KeyEvent::from).collect();
                let mut read = read.iter();

                let mut write: Vec<u8> = Vec::new();
                let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
                let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

                let ans = $prompt.prompt_with_backend(&mut backend).unwrap();

                assert_eq!($output, ans);
            }
        };
    }

    password_test!(empty, vec![KeyCode::Enter], "");

    password_test!(single_letter, vec![KeyCode::Char('b'), KeyCode::Enter], "b");

    password_test!(
        letters_and_enter,
        text_to_events!("normal input\n"),
        "normal input"
    );

    password_test!(
        letters_and_enter_with_emoji,
        text_to_events!("with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞\n"),
        "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞"
    );

    password_test!(
        input_and_correction,
        {
            let mut events = vec![];
            events.append(&mut text_to_events!("anor").collect());
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.append(&mut text_to_events!("normal input").collect());
            events.push(KeyCode::Enter);
            events
        },
        "normal input"
    );

    password_test!(
        input_and_excessive_correction,
        {
            let mut events = vec![];
            events.append(&mut text_to_events!("anor").collect());
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.append(&mut text_to_events!("normal input").collect());
            events.push(KeyCode::Enter);
            events
        },
        "normal input"
    );

    password_test!(
        input_correction_after_validation,
        {
            let mut events = vec![];
            events.append(&mut text_to_events!("1234567890").collect());
            events.push(KeyCode::Enter);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.append(&mut text_to_events!("yes").collect());
            events.push(KeyCode::Enter);
            events
        },
        "12345yes",
        Password::new("").with_validator(&|ans| match ans.len() {
            len if len > 5 && len < 10 => Ok(()),
            _ => Err("Invalid".to_string()),
        })
    );
}
