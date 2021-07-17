use unicode_segmentation::UnicodeSegmentation;

use crate::{
    error::{InquireError, InquireResult},
    formatter::{BoolFormatter, DEFAULT_BOOL_FORMATTER},
    key::{Key, KeyModifiers},
    parser::{BoolParser, DEFAULT_BOOL_PARSER},
    renderer::Renderer,
    terminal::Terminal,
};

/// Presents a message to the user and asks them for a yes/no confirmation.
#[derive(Copy, Clone)]
pub struct Confirm<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Default value, returned when the user input is empty.
    pub default: Option<bool>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: BoolFormatter<'a>,

    /// Function that parses the user input and returns the result
    pub parser: BoolParser<'a>,

    /// Function that formats the default value to be presented to the user
    pub default_value_formatter: BoolFormatter<'a>,
}

impl<'a> Confirm<'a> {
    /// Default formatter, [true] maps to "Yes" and [false] maps to "No".
    pub const DEFAULT_FORMATTER: BoolFormatter<'a> = DEFAULT_BOOL_FORMATTER;
    /// Default parser, matches ["y"] and ["yes"] to [true], ["n"] and ["no"]
    /// to [false], and an [Err] otherwise.
    pub const DEFAULT_PARSER: BoolParser<'a> = DEFAULT_BOOL_PARSER;

    /// Default formatter for default values, mapping [true] to ["Y/n"] and
    /// [false] to ["y/N"]
    pub const DEFAULT_DEFAULT_VALUE_FORMATTER: BoolFormatter<'a> = |ans| match ans {
        true => "Y/n",
        false => "y/N",
    };

    /// Creates a [Confirm] with the provided message and default configuration values.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            default: None,
            help_message: None,
            formatter: Self::DEFAULT_FORMATTER,
            parser: Self::DEFAULT_PARSER,
            default_value_formatter: Self::DEFAULT_DEFAULT_VALUE_FORMATTER,
        }
    }

    /// Sets the default input.
    pub fn with_default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Sets the formatter
    pub fn with_formatter(mut self, formatter: BoolFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the parser
    pub fn with_parser(mut self, parser: BoolParser<'a>) -> Self {
        self.parser = parser;
        self
    }

    /// Sets the default value formatter
    pub fn with_default_value_formatter(mut self, formatter: BoolFormatter<'a>) -> Self {
        self.default_value_formatter = formatter;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to them.
    pub fn prompt(self) -> InquireResult<bool> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(self, renderer: &mut Renderer) -> InquireResult<bool> {
        ConfirmPrompt::from(self).prompt(renderer)
    }
}

impl<'a> From<&'a str> for Confirm<'a> {
    fn from(val: &'a str) -> Self {
        Confirm::new(val)
    }
}

struct ConfirmPrompt<'a> {
    message: &'a str,
    error: Option<String>,
    help_message: Option<&'a str>,
    default: Option<bool>,
    content: String,
    formatter: BoolFormatter<'a>,
    parser: BoolParser<'a>,
    default_value_formatter: BoolFormatter<'a>,
}

impl<'a> From<Confirm<'a>> for ConfirmPrompt<'a> {
    fn from(co: Confirm<'a>) -> Self {
        Self {
            message: co.message,
            error: None,
            default: co.default,
            help_message: co.help_message,
            formatter: co.formatter,
            parser: co.parser,
            default_value_formatter: co.default_value_formatter,
            content: String::new(),
        }
    }
}

impl<'a> ConfirmPrompt<'a> {
    fn on_change(&mut self, key: Key) {
        match key {
            Key::Backspace => {
                let len = self.content[..].graphemes(true).count();
                let new_len = len.saturating_sub(1);
                self.content = self.content[..].graphemes(true).take(new_len).collect();
            }
            Key::Char(c, KeyModifiers::NONE) => self.content.push(c),
            _ => {}
        }
    }

    fn get_final_answer(&self) -> Result<bool, String> {
        match self.default {
            Some(val) if self.content.is_empty() => return Ok(val),
            _ => {}
        }

        (self.parser)(&self.content)
    }

    fn render(&mut self, renderer: &mut Renderer) -> InquireResult<()> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(error_message) = &self.error {
            renderer.print_error_message(error_message)?;
        }

        let default_message = self.default.map(self.default_value_formatter);

        renderer.print_prompt(&prompt, default_message, Some(&self.content))?;

        if let Some(message) = self.help_message {
            renderer.print_help(message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> InquireResult<bool> {
        let final_answer: bool;

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
                    Err(message) => {
                        self.error = Some(message);
                        self.content.clear();
                    }
                },
                key => self.on_change(key),
            }
        }

        let formatted = (self.formatter)(final_answer);

        renderer.cleanup(&self.message, &formatted)?;

        Ok(final_answer)
    }
}
