mod action;
#[cfg(test)]
mod test;

pub use action::*;

use crate::{
    config::get_configuration,
    error::{InquireError, InquireResult},
    formatter::{BoolFormatter, DEFAULT_BOOL_FORMATTER},
    parser::{BoolParser, DEFAULT_BOOL_PARSER},
    terminal::get_default_terminal,
    ui::{Backend, CustomTypeBackend, RenderConfig},
    CustomType,
};

/// Prompt to ask the user for simple yes/no questions, commonly known by asking the user displaying the `(y/n)` text.
///
/// This prompt is basically a wrapper around the behavior of `CustomType` prompts, providing a sensible set of defaults to ask for simple `true/false` questions, such as confirming an action.
///
/// Default values are formatted with the given value in uppercase, e.g. `(Y/n)` or `(y/N)`. The `bool` parser accepts by default only the following inputs (case-insensitive): `y`, `n`, `yes` and `no`. If the user input does not match any of them, the following error message is displayed by default:
/// - `# Invalid answer, try typing 'y' for yes or 'n' for no`.
///
/// Finally, once the answer is submitted, [`Confirm`] prompts display the bool value formatted as either "Yes", if a `true` value was parsed, or "No" otherwise.
///
/// The Confirm prompt does not support custom validators because of the nature of the prompt. The user input is always parsed to true or false. If one of the two alternatives is invalid, a Confirm prompt that only allows yes or no answers does not make a lot of sense to me, but if someone provides a clear use-case I will reconsider.
///
/// Confirm prompts provide several options of configuration:
///
/// - **Prompt message**: Required when creating the prompt.
/// - **Default value**: Default value returned when the user submits an empty response.
/// - **Placeholder**: Short hint that describes the expected value of the input.
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
///   - Formats `true` to "Yes" and `false` to "No", by default.
/// - **Parser**: Custom parser for user inputs.
///   - The default `bool` parser returns `true` if the input is either `"y"` or `"yes"`, in a case-insensitive comparison. Similarly, the parser returns `false` if the input is either `"n"` or `"no"`.
/// - **Default value formatter**: Function that formats how the default value is displayed to the user.
///   - By default, displays "y/n" with the default value capitalized, e.g. "y/N".
/// - **Error message**: Error message to display when a value could not be parsed from the input.
///   - Set to "Invalid answer, try typing 'y' for yes or 'n' for no" by default.
///
/// # Example
///
/// ```no_run
/// use inquire::Confirm;
///
/// let ans = Confirm::new("Do you live in Brazil?")
///     .with_default(false)
///     .with_help_message("This data is stored for good reasons")
///     .prompt();
///
/// match ans {
///     Ok(true) => println!("That's awesome!"),
///     Ok(false) => println!("That's too bad, I've heard great things about it."),
///     Err(_) => println!("Error with questionnaire, try again later"),
/// }
/// ```
///
/// [`Confirm`]: crate::Confirm
#[derive(Clone)]
pub struct Confirm<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Initial value of the prompt's text input.
    ///
    /// If you want to set a default value for the prompt, returned when the user's submission is empty, see [`default`].
    ///
    /// [`default`]: Self::default
    pub starting_input: Option<&'a str>,

    /// Default value, returned when the user input is empty.
    pub default: Option<bool>,

    /// Short hint that describes the expected value of the input.
    pub placeholder: Option<&'a str>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: BoolFormatter<'a>,

    /// Function that parses the user input and returns the result value.
    pub parser: BoolParser<'a>,

    /// Function that formats the default value to be presented to the user
    pub default_value_formatter: BoolFormatter<'a>,

    /// Error message displayed when a value could not be parsed from input.
    pub error_message: String,

    /// RenderConfig to apply to the rendered interface.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still support NO_COLOR, you will have to do this on your end.
    pub render_config: RenderConfig<'a>,
}

impl<'a> Confirm<'a> {
    /// Default formatter, set to [DEFAULT_BOOL_FORMATTER](crate::formatter::DEFAULT_BOOL_FORMATTER)
    pub const DEFAULT_FORMATTER: BoolFormatter<'a> = DEFAULT_BOOL_FORMATTER;

    /// Default input parser.
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
            starting_input: None,
            default: None,
            placeholder: None,
            help_message: None,
            formatter: Self::DEFAULT_FORMATTER,
            parser: Self::DEFAULT_PARSER,
            default_value_formatter: Self::DEFAULT_DEFAULT_VALUE_FORMATTER,
            error_message: String::from(Self::DEFAULT_ERROR_MESSAGE),
            render_config: get_configuration(),
        }
    }

    /// Sets the initial value of the prompt's text input.
    ///
    /// If you want to set a default value for the prompt, returned when the user's submission is empty, see [`with_default`].
    ///
    /// [`with_default`]: Self::with_default
    pub fn with_starting_input(mut self, message: &'a str) -> Self {
        self.starting_input = Some(message);
        self
    }

    /// Sets the default input.
    pub fn with_default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets the placeholder.
    pub fn with_placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: BoolFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the parser.
    pub fn with_parser(mut self, parser: BoolParser<'a>) -> Self {
        self.parser = parser;
        self
    }

    /// Sets a custom error message displayed when a submission could not be parsed to a value.
    pub fn with_error_message(mut self, error_message: &'a str) -> Self {
        self.error_message = String::from(error_message);
        self
    }

    /// Sets the default value formatter
    pub fn with_default_value_formatter(mut self, formatter: BoolFormatter<'a>) -> Self {
        self.default_value_formatter = formatter;
        self
    }

    /// Sets the provided color theme to this prompt.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still support NO_COLOR, you will have to do this on your end.
    pub fn with_render_config(mut self, render_config: RenderConfig<'a>) -> Self {
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
    pub fn prompt_skippable(self) -> InquireResult<Option<bool>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<bool> {
        let (input_reader, terminal) = get_default_terminal()?;
        let mut backend = Backend::new(input_reader, terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(crate) fn prompt_with_backend<B: CustomTypeBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<bool> {
        CustomType::from(self).prompt_with_backend(backend)
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
            starting_input: co.starting_input,
            default: co.default,
            default_value_formatter: co.default_value_formatter,
            placeholder: co.placeholder,
            help_message: co.help_message,
            formatter: co.formatter,
            parser: co.parser,
            validators: vec![],
            error_message: co.error_message,
            render_config: co.render_config,
        }
    }
}
