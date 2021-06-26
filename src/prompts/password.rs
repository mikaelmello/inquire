use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    formatter::StringFormatter, renderer::Renderer, terminal::Terminal, validator::StringValidator,
};

const DEFAULT_FORMATTER: StringFormatter = |_| "********";

#[derive(Clone)]
pub struct Password<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    formatter: StringFormatter,
    validator: Option<StringValidator>,
}

impl<'a> Password<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            help_message: None,
            formatter: DEFAULT_FORMATTER,
            validator: None,
        }
    }

    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    pub fn with_formatter(mut self, formatter: StringFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn with_validator(mut self, validator: StringValidator) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn prompt(self) -> Result<String, Box<dyn Error>> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(
        self,
        renderer: &mut Renderer,
    ) -> Result<String, Box<dyn Error>> {
        PasswordPrompt::from(self).prompt(renderer)
    }
}

struct PasswordPrompt<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    content: String,
    formatter: StringFormatter,
    validator: Option<StringValidator>,
    error: Option<String>,
}

impl<'a> From<Password<'a>> for PasswordPrompt<'a> {
    fn from(so: Password<'a>) -> Self {
        Self {
            message: so.message,
            help_message: so.help_message,
            formatter: so.formatter,
            validator: so.validator,
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
        if let Some(validator) = self.validator {
            match validator(&self.content) {
                Ok(_) => {}
                Err(err) => return Err(err.to_string()),
            }
        }

        Ok(self.content.clone())
    }

    fn render(&mut self, renderer: &mut Renderer) -> Result<(), std::io::Error> {
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

    fn prompt(mut self, renderer: &mut Renderer) -> Result<String, Box<dyn Error>> {
        let final_answer: String;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Ctrl('c') => bail!("Password input interrupted by ctrl-c"),
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
                let terminal = Terminal::new_with_io(&mut write, &mut read).unwrap();
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
            _ => Err("Invalid"),
        })
    );
}
