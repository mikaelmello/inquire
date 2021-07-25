use crate::{
    error::InquireResult,
    formatter::{BoolFormatter, DEFAULT_BOOL_FORMATTER},
    parser::{BoolParser, DEFAULT_BOOL_PARSER},
    renderer::Renderer,
    terminal::Terminal,
    CustomType,
};

/// Presents a message to the user and asks them for a yes/no confirmation.
#[derive(Clone)]
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

    /// Error message displayed when value could not be parsed from input.
    pub error_message: String,
}

impl<'a> Confirm<'a> {
    /// Default formatter, [true] maps to "Yes" and [false] maps to "No".
    pub const DEFAULT_FORMATTER: BoolFormatter<'a> = DEFAULT_BOOL_FORMATTER;
    /// Default parser, matches ["y"] and ["yes"] to [true], ["n"] and ["no"]
    /// to [false], and an [Err] otherwise.
    pub const DEFAULT_PARSER: BoolParser<'a> = DEFAULT_BOOL_PARSER;

    /// Default formatter for default values, mapping [true] to ["Y/n"] and
    /// [false] to ["y/N"]
    pub const DEFAULT_DEFAULT_VALUE_FORMATTER: BoolFormatter<'a> = &|ans| match ans {
        true => String::from("Y/n"),
        false => String::from("y/N"),
    };

    /// Default error message displayed when parsing fails.
    pub const DEFAULT_ERROR_MESSAGE: &'a str =
        "Invalid answer, try typing 'y' for yes or 'n' for no";

    /// Creates a [Confirm] with the provided message and default configuration values.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            default: None,
            help_message: None,
            formatter: Self::DEFAULT_FORMATTER,
            parser: Self::DEFAULT_PARSER,
            default_value_formatter: Self::DEFAULT_DEFAULT_VALUE_FORMATTER,
            error_message: String::from(Self::DEFAULT_ERROR_MESSAGE),
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
        CustomType::from(self).prompt_with_renderer(renderer)
    }
}

impl<'a> From<&'a str> for Confirm<'a> {
    fn from(val: &'a str) -> Self {
        Confirm::new(val)
    }
}

impl<'a> From<Confirm<'a>> for CustomType<'a, bool> {
    fn from(co: Confirm<'a>) -> Self {
        Self {
            message: co.message,
            default: match co.default {
                Some(val) => Some((val, co.default_value_formatter)),
                None => None,
            },
            help_message: co.help_message,
            formatter: co.formatter,
            parser: co.parser,
            error_message: co.error_message,
        }
    }
}
