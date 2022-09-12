use std::str::FromStr;

use crate::{
    config::get_configuration,
    error::{InquireError, InquireResult},
    formatter::CustomTypeFormatter,
    input::Input,
    parser::CustomTypeParser,
    terminal::get_default_terminal,
    ui::{Backend, CustomTypeBackend, Key, RenderConfig},
    validator::{CustomTypeValidator, ErrorMessage, Validation},
};

/// Generic prompt suitable for when you need to parse the user input into a specific type, for example an `f64` or a `rust_decimal`, maybe even an `uuid`.
///
/// This prompt has all of the validation, parsing and error handling features built-in to reduce as much boilerplaste as possible from your prompts. Its defaults are necessarily very simple in order to cover a large range of generic cases, for example a "Invalid input" error message.
///
/// You can customize as many aspects of this prompt as you like: prompt message, help message, default value, placeholder, value parser and value formatter.
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
/// use inquire::{CustomType, ui::RenderConfig};
///
/// let amount_prompt: CustomType<f64> = CustomType {
///     message: "How much is your travel going to cost?",
///     formatter: &|i| format!("${:.2}", i),
///     default_value_formatter: &|i| format!("${:.2}", i),
///     default: None,
///     validators: vec![],
///     placeholder: Some("123.45"),
///     error_message: "Please type a valid number.".into(),
///     help_message: "Do not use currency and the number should use dots as the decimal separator.".into(),
///     parser: &|i| match i.parse::<f64>() {
///         Ok(val) => Ok(val),
///         Err(_) => Err(()),
///     },
///     render_config: RenderConfig::default(),
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
    pub default: Option<T>,

    /// Short hint that describes the expected value of the input.
    pub placeholder: Option<&'a str>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: CustomTypeFormatter<'a, T>,

    /// Function that formats the provided value. Useful for example when you want to format a default `true` to the string "Y/n", common in confirmation prompts.
    pub default_value_formatter: CustomTypeFormatter<'a, T>,

    /// Function that parses the user input and returns the result value.
    pub parser: CustomTypeParser<'a, T>,

    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Vec<Box<dyn CustomTypeValidator<T>>>,

    /// Error message displayed when value could not be parsed from input.
    pub error_message: String,

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

impl<'a, T> CustomType<'a, T>
where
    T: Clone,
{
    /// Default validators added to the [CustomType] prompt, none.
    pub const DEFAULT_VALIDATORS: Vec<Box<dyn CustomTypeValidator<T>>> = vec![];

    /// Creates a [CustomType] with the provided message and default configuration values.
    pub fn new(message: &'a str) -> Self
    where
        T: FromStr + ToString,
    {
        Self {
            message,
            default: None,
            placeholder: None,
            help_message: None,
            formatter: &|val| val.to_string(),
            default_value_formatter: &|val| val.to_string(),
            parser: &|a| a.parse::<T>().map_err(|_| ()),
            validators: Self::DEFAULT_VALIDATORS,
            error_message: "Invalid input".into(),
            render_config: get_configuration(),
        }
    }

    /// Sets the default input.
    pub fn with_default(mut self, default: T) -> Self {
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

    /// Sets the formatter
    pub fn with_formatter(mut self, formatter: CustomTypeFormatter<'a, T>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the formatter for default values.
    ///
    /// Useful for example when you want to format a default `true` to the string "Y/n", common in confirmation prompts,
    /// when the final answer would be displayed likely as "Yes" or "No".
    pub fn with_default_value_formatter(mut self, formatter: CustomTypeFormatter<'a, T>) -> Self {
        self.default_value_formatter = formatter;
        self
    }

    /// Sets the parser.
    pub fn with_parser(mut self, parser: CustomTypeParser<'a, T>) -> Self {
        self.parser = parser;
        self
    }

    /// Adds a validator to the collection of validators. You might want to use this feature
    /// in case you need to require certain features from the parsed user's answer.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validator<V>(mut self, validator: V) -> Self
    where
        V: CustomTypeValidator<T> + 'static,
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
    /// from the parsed user's answer.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validators(mut self, validators: &[Box<dyn CustomTypeValidator<T>>]) -> Self {
        for validator in validators {
            #[allow(clippy::clone_double_ref)]
            self.validators.push(validator.clone());
        }
        self
    }

    /// Sets a custom error message displayed when a submission could not be parsed to a value.
    pub fn with_error_message(mut self, error_message: &'a str) -> Self {
        self.error_message = String::from(error_message);
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
    pub fn prompt_skippable(self) -> InquireResult<Option<T>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<T> {
        let terminal = get_default_terminal()?;
        let mut backend = Backend::new(terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(crate) fn prompt_with_backend<B: CustomTypeBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<T> {
        CustomTypePrompt::from(self).prompt(backend)
    }
}

struct CustomTypePrompt<'a, T> {
    message: &'a str,
    error: Option<ErrorMessage>,
    help_message: Option<&'a str>,
    default: Option<T>,
    input: Input,
    formatter: CustomTypeFormatter<'a, T>,
    default_value_formatter: CustomTypeFormatter<'a, T>,
    validators: Vec<Box<dyn CustomTypeValidator<T>>>,
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
            default_value_formatter: co.default_value_formatter,
            validators: co.validators,
            parser: co.parser,
            input: co
                .placeholder
                .map(|p| Input::new().with_placeholder(p))
                .unwrap_or_else(Input::new),
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

    fn validate_current_answer(&self, value: &T) -> InquireResult<Validation> {
        for validator in &self.validators {
            match validator.validate(value) {
                Ok(Validation::Valid) => {}
                Ok(Validation::Invalid(msg)) => return Ok(Validation::Invalid(msg)),
                Err(err) => return Err(InquireError::Custom(err)),
            }
        }

        Ok(Validation::Valid)
    }

    fn get_final_answer(&self) -> Result<T, String> {
        match &self.default {
            Some(val) if self.input.content().is_empty() => return Ok(val.clone()),
            _ => {}
        }

        match (self.parser)(self.input.content()) {
            Ok(val) => Ok(val),
            Err(_) => Err(self.error_message.clone()),
        }
    }

    fn render<B: CustomTypeBackend>(&mut self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        backend.frame_setup()?;

        if let Some(error) = &self.error {
            backend.render_error_message(error)?;
        }

        let default_value_formatter = self.default_value_formatter;
        let default_message = self
            .default
            .as_ref()
            .map(|val| default_value_formatter(val.clone()));

        backend.render_prompt(prompt, default_message.as_deref(), &self.input)?;

        if let Some(message) = self.help_message {
            backend.render_help_message(message)?;
        }

        backend.frame_finish()?;

        Ok(())
    }

    fn prompt<B: CustomTypeBackend>(mut self, backend: &mut B) -> InquireResult<T> {
        let final_answer: T;

        loop {
            self.render(backend)?;

            let key = backend.read_key()?;

            match key {
                Key::Interrupt => interrupt_prompt!(),
                Key::Cancel => cancel_prompt!(backend, self.message),
                Key::Submit => match self.get_final_answer() {
                    Ok(answer) => match self.validate_current_answer(&answer)? {
                        Validation::Valid => {
                            final_answer = answer;
                            break;
                        }
                        Validation::Invalid(msg) => self.error = Some(msg),
                    },
                    Err(message) => self.error = Some(message.into()),
                },
                key => self.on_change(key),
            }
        }

        let formatted = (self.formatter)(final_answer.clone());

        finish_prompt_with_answer!(backend, self.message, &formatted, final_answer);
    }
}
