use crate::{
    error::{InquireError, InquireResult},
    formatter::StringFormatter,
    input::Input,
    renderer::Renderer,
    terminal::Terminal,
    ui::key::Key,
    validator::StringValidator,
};

/// Prompt meant for secretive text inputs.
///
/// It is a simple text prompt where the user's input is captured and not echoed back to the terminal.
///
/// This prompt is meant to be as simple and raw as possible, not supporting features such as default values or auto-completion.
///
/// By default, the user submission is formatted as "\*\*\*\*\*\*\*\*" (eight star characters).
///
/// This prompt still allows the caller to customize standard properties: validators, input formatter, error and help messages.
///
/// # Example
///
/// ```no_run
/// use inquire::Password;
///
/// let name = Password::new("Encryption key:").prompt();
///
/// match name {
///     Ok(_) => println!("This doesn't look like a key."),
///     Err(_) => println!("An error happened when asking for your key, try again later."),
/// }
/// ```
#[derive(Clone)]
pub struct Password<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

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
    pub validators: Vec<StringValidator<'a>>,
}

impl<'a> Password<'a> {
    /// Default formatter, set to always display `"********"` regardless of input length.
    pub const DEFAULT_FORMATTER: StringFormatter<'a> = &|_| String::from("********");

    /// Default validators added to the [Password] prompt, none.
    pub const DEFAULT_VALIDATORS: Vec<StringValidator<'a>> = vec![];

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> = None;

    /// Creates a [Password] with the provided message and default options.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            formatter: Self::DEFAULT_FORMATTER,
            validators: Self::DEFAULT_VALIDATORS,
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
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
            self.validators.push(validator.clone());
        }
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<String> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(self, renderer: &mut Renderer) -> InquireResult<String> {
        PasswordPrompt::from(self).prompt(renderer)
    }
}

struct PasswordPrompt<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    input: Input,
    formatter: StringFormatter<'a>,
    validators: Vec<StringValidator<'a>>,
    error: Option<String>,
}

impl<'a> From<Password<'a>> for PasswordPrompt<'a> {
    fn from(so: Password<'a>) -> Self {
        Self {
            message: so.message,
            help_message: so.help_message,
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
        self.input.handle_key(key);
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

    fn render(&mut self, renderer: &mut Renderer) -> InquireResult<()> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(err) = &self.error {
            renderer.print_error_message(err)?;
        }

        renderer.print_prompt(&prompt, None, None)?;

        if let Some(message) = self.help_message {
            renderer.print_help(message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> InquireResult<String> {
        let final_answer: String;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Cancel => return Err(InquireError::OperationCanceled),
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

        renderer.cleanup(&self.message, &(self.formatter)(&final_answer))?;

        Ok(final_answer)
    }
}

#[cfg(test)]
mod test {
    use super::Password;
    use crate::{renderer::Renderer, terminal::Terminal};
    use crossterm::event::{KeyCode, KeyEvent};
    use ntest::timeout;

    fn default<'a>() -> Password<'a> {
        Password::new("Question?")
    }

    macro_rules! text_to_events {
        ($text:expr) => {{
            $text.chars().map(|c| KeyCode::Char(c))
        }};
    }

    macro_rules! password_test {
        ($name:ident,$input:expr,$output:expr) => {
            password_test! {$name, $input, $output, default()}
        };

        ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
            #[test]
            #[timeout(100)]
            fn $name() {
                let read: Vec<KeyEvent> = $input.into_iter().map(KeyEvent::from).collect();
                let mut read = read.iter();

                let mut write: Vec<u8> = Vec::new();
                let terminal = Terminal::new_with_io(&mut write, &mut read);
                let mut renderer = Renderer::new(terminal).unwrap();

                let ans = $prompt.prompt_with_renderer(&mut renderer).unwrap();

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
