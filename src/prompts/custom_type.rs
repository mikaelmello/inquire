use std::str::FromStr;

use crate::{
    error::{InquireError, InquireResult},
    formatter::CustomTypeFormatter,
    input::Input,
    key::Key,
    parse_type,
    parser::CustomTypeParser,
    renderer::Renderer,
    terminal::Terminal,
};

/// [`CustomType`] prompts are generic prompts suitable for when you need to parse the user input into a specific type, for example an `f64` or a `rust_decimal`, maybe even an `uuid`.
///
/// This prompt has all of the validation, parsing and error handling features built-in to reduce as much boilerplaste as possible from your prompts. Its defaults are necessarily very simple in order to cover a large range of generic cases, for example a "Invalid input" error message.
///
/// You can customize as many aspects of this prompt as you like: prompt message, help message, default value, value parser and value formatter.
///
/// # Behavior
///
/// When initializing this prompt via the `new()` method, some constraints on the return type `T` are added to make sure we can apply a default parser and formatter to the prompt.
///
/// The default parser calls the [`str.parse`](https://doc.rust-lang.org/stable/std/primitive.str.html#method.parse) method, which means that `T` must implement the `FromStr` trait. When the parsing fails for any reason, a default error message "Invalid input" is displayed to the user.
///
/// After the user submits, the prompt handler tries to parse the input into the expected type. If the operation succeeds, the value is returned to the prompt caller. If it fails, the message defined in `error_message` is displayed to the user.
///
/// The default formatter simply calls `to_string()` on the parsed value, which means that `T` must implement the `ToString` trait, which normally happens implicitly when you implement the `Display` trait.
///
/// If your type `T` does not satisfy these constraints, you can always manually instantiate the entire struct yourself like this:
///
/// ```no_run
/// use inquire::CustomType;
///
/// let amount_prompt: CustomType<f64> = CustomType {
///     message: "How much is your travel going to cost?",
///     formatter: &|i| format!("${:.2}", i),
///     default: None,
///     error_message: "Please type a valid number.".into(),
///     help_message: "Do not use currency and the number should use dots as the decimal separator.".into(),
///     parser: &|i| match i.parse::<f64>() {
///         Ok(val) => Ok(val),
///         Err(_) => Err(()),
///     },
/// };
/// ```
///
/// # Example
///
/// ```no_run
/// use inquire::CustomType;
///
/// let amount = CustomType::<f64>::new("How much do you want to donate?")
///     .with_formatter(&|i| format!("${:.2}", i))
///     .with_error_message("Please type a valid number")
///     .with_help_message("Type the amount in US dollars using a decimal point as a separator")
///     .prompt();
///
/// match amount {
///     Ok(_) => println!("Thanks a lot for donating that much money!"),
///     Err(_) => println!("We could not process your donation"),
/// }
/// ```
///
/// [`CustomType`]: crate::CustomType
#[derive(Clone)]
pub struct CustomType<'a, T> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Default value, returned when the user input is empty.
    pub default: Option<(T, CustomTypeFormatter<'a, T>)>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: CustomTypeFormatter<'a, T>,

    /// Function that parses the user input and returns the result value.
    pub parser: CustomTypeParser<'a, T>,

    /// Error message displayed when value could not be parsed from input.
    pub error_message: String,
}

impl<'a, T> CustomType<'a, T>
where
    T: Clone,
{
    /// Creates a [CustomType] with the provided message and default configuration values.
    pub fn new(message: &'a str) -> Self
    where
        T: FromStr + ToString,
    {
        Self {
            message,
            default: None,
            help_message: None,
            formatter: &|val| val.to_string(),
            parser: parse_type!(T),
            error_message: "Invalid input".into(),
        }
    }

    /// Sets the default input.
    pub fn with_default(mut self, default: (T, CustomTypeFormatter<'a, T>)) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Sets the formatter
    pub fn with_formatter(mut self, formatter: CustomTypeFormatter<'a, T>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the parser.
    pub fn with_parser(mut self, parser: CustomTypeParser<'a, T>) -> Self {
        self.parser = parser;
        self
    }

    /// Sets a custom error message displayed when a submission could not be parsed to a value.
    pub fn with_error_message(mut self, error_message: &'a str) -> Self {
        self.error_message = String::from(error_message);
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<T> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(self, renderer: &mut Renderer) -> InquireResult<T> {
        CustomTypePrompt::from(self).prompt(renderer)
    }
}

struct CustomTypePrompt<'a, T> {
    message: &'a str,
    error: Option<String>,
    help_message: Option<&'a str>,
    default: Option<(T, CustomTypeFormatter<'a, T>)>,
    input: Input,
    formatter: CustomTypeFormatter<'a, T>,
    parser: CustomTypeParser<'a, T>,
    error_message: String,
}

impl<'a, T> From<CustomType<'a, T>> for CustomTypePrompt<'a, T>
where
    T: Clone,
{
    fn from(co: CustomType<'a, T>) -> Self {
        Self {
            message: co.message,
            error: None,
            default: co.default,
            help_message: co.help_message,
            formatter: co.formatter,
            parser: co.parser,
            input: Input::new(),
            error_message: co.error_message,
        }
    }
}

impl<'a, T> CustomTypePrompt<'a, T>
where
    T: Clone,
{
    fn on_change(&mut self, key: Key) {
        self.input.handle_key(key);
    }

    fn get_final_answer(&self) -> Result<T, String> {
        match &self.default {
            Some((val, _)) if self.input.content().is_empty() => return Ok(val.clone()),
            _ => {}
        }

        match (self.parser)(self.input.content()) {
            Ok(val) => Ok(val),
            Err(_) => Err(self.error_message.clone()),
        }
    }

    fn render(&mut self, renderer: &mut Renderer) -> InquireResult<()> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(error_message) = &self.error {
            renderer.print_error_message(error_message)?;
        }

        let default_message = self
            .default
            .as_ref()
            .map(|(val, formatter)| formatter(val.clone()));

        renderer.print_prompt_input(&prompt, default_message.as_deref(), &self.input)?;

        if let Some(message) = self.help_message {
            renderer.print_help(message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> InquireResult<T> {
        let final_answer: T;

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
                        self.input.clear();
                    }
                },
                key => self.on_change(key),
            }
        }

        let formatted = (self.formatter)(final_answer.clone());

        renderer.cleanup(&self.message, &formatted)?;

        Ok(final_answer)
    }
}
