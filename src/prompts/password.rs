use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    error::{InquireError, InquireResult},
    formatter::StringFormatter,
    renderer::Renderer,
    terminal::Terminal,
    validator::StringValidator,
};
/// Presents a message to the user and retrieves a single line of text input.
///
/// [Password] differs from [Text] by not echoing the user input and having
/// a smaller set of custom behaviors in comparison.
///
/// By default, the response is always formatted as "********".
#[derive(Clone)]
pub struct Password<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: StringFormatter,

    /// Collection of validators to apply to the user input.
    /// Validation errors are displayed to the user one line above the prompt.
    pub validators: Vec<StringValidator>,
}

impl<'a> Password<'a> {
    /// Default formatter.
    pub const DEFAULT_FORMATTER: StringFormatter = |_| "********";
    /// Default collection of validators.
    pub const DEFAULT_VALIDATORS: Vec<StringValidator> = Vec::new();
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

    /// Sets the formatter
    pub fn with_formatter(mut self, formatter: StringFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    /// Adds a validator to the collection of validators.
    pub fn with_validator(mut self, validator: StringValidator) -> Self {
        self.validators.push(validator);
        self
    }

    /// Adds the validators to the collection of validators.
    pub fn with_validators(mut self, validators: &[StringValidator]) -> Self {
        for validator in validators {
            self.validators.push(validator.clone());
        }
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to them.
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
    content: String,
    formatter: StringFormatter,
    validators: Vec<StringValidator>,
    error: Option<String>,
}

impl<'a> From<Password<'a>> for PasswordPrompt<'a> {
    fn from(so: Password<'a>) -> Self {
        Self {
            message: so.message,
            help_message: so.help_message,
            formatter: so.formatter,
            validators: so.validators,
            content: String::new(),
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
            Key::Backspace => {
                let len = self.content[..].graphemes(true).count();
                let new_len = len.saturating_sub(1);
                self.content = self.content[..].graphemes(true).take(new_len).collect();
            }
            Key::Char(c) => self.content.push(c),
            _ => {}
        }
    }

    fn get_final_answer(&self) -> Result<String, String> {
        for validator in &self.validators {
            match validator(&self.content) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(self.content.clone())
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
                Key::Ctrl('c') => return Err(InquireError::OperationCanceled),
                Key::Char('\n') | Key::Char('\r') => match self.get_final_answer() {
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
    use ntest::timeout;

    use crate::{renderer::Renderer, terminal::Terminal, Password};

    fn default<'a>() -> Password<'a> {
        Password::new("Question?")
    }

    macro_rules! password_test {
        ($name:ident,$input:expr,$output:expr) => {
            password_test! {$name, $input, $output, default()}
        };

        ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
            #[test]
            #[timeout(100)]
            fn $name() {
                let mut read: &[u8] = $input.as_bytes();

                let mut write: Vec<u8> = Vec::new();
                let terminal = Terminal::new_with_io(&mut write, &mut read);
                let mut renderer = Renderer::new(terminal).unwrap();

                let ans = $prompt.prompt_with_renderer(&mut renderer).unwrap();

                assert_eq!($output, ans);
            }
        };
    }

    password_test!(empty, "\n", "");

    password_test!(single_letter, "b\n", "b");

    password_test!(letters_and_enter, "normal input\n", "normal input");

    password_test!(
        letters_and_enter_with_emoji,
        "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞\n",
        "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞"
    );

    password_test!(
        input_and_correction,
        "anor\x7F\x7F\x7F\x7Fnormal input\n",
        "normal input"
    );

    password_test!(
        input_and_excessive_correction,
        "anor\x7F\x7F\x7F\x7F\x7F\x7F\x7F\x7Fnormal input\n",
        "normal input"
    );

    password_test!(
        input_correction_after_validation,
        "1234567890\n\x7F\x7F\x7F\x7F\x7F\nyes\n",
        "12345yes",
        Password::new("").with_validator(|ans| match ans.len() {
            len if len > 5 && len < 10 => Ok(()),
            _ => Err("Invalid".to_string()),
        })
    );
}
