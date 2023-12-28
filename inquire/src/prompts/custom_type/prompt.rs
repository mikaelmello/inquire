use crate::{
    error::InquireResult,
    formatter::CustomTypeFormatter,
    input::Input,
    parser::CustomTypeParser,
    prompts::prompt::{ActionResult, Prompt},
    ui::CustomTypeBackend,
    validator::{CustomTypeValidator, ErrorMessage, Validation},
    CustomType, InquireError,
};

use super::{action::CustomTypePromptAction, config::CustomTypeConfig};

pub struct CustomTypePrompt<'a, T> {
    message: &'a str,
    config: CustomTypeConfig,
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
        let input = Input::new_with(co.starting_input.unwrap_or_default());
        let input = if let Some(placeholder) = co.placeholder {
            input.with_placeholder(placeholder)
        } else {
            input
        };

        Self {
            message: co.message,
            config: (&co).into(),
            error: None,
            default: co.default,
            help_message: co.help_message,
            formatter: co.formatter,
            default_value_formatter: co.default_value_formatter,
            validators: co.validators,
            parser: co.parser,
            input,
            error_message: co.error_message,
        }
    }
}

impl<'a, T> CustomTypePrompt<'a, T>
where
    T: Clone,
{
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
}

impl<'a, Backend, T> Prompt<Backend> for CustomTypePrompt<'a, T>
where
    Backend: CustomTypeBackend,
    T: Clone,
{
    type Config = CustomTypeConfig;
    type InnerAction = CustomTypePromptAction;
    type Output = T;

    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &CustomTypeConfig {
        &self.config
    }

    fn format_answer(&self, answer: &T) -> String {
        (self.formatter)((*answer).clone())
    }

    fn submit(&mut self) -> InquireResult<Option<T>> {
        let answer = match self.get_final_answer() {
            Ok(answer) => match self.validate_current_answer(&answer)? {
                Validation::Valid => Some(answer),
                Validation::Invalid(msg) => {
                    self.error = Some(msg);
                    None
                }
            },
            Err(message) => {
                self.error = Some(message.into());
                None
            }
        };

        Ok(answer)
    }

    fn handle(&mut self, action: CustomTypePromptAction) -> InquireResult<ActionResult> {
        let result = match action {
            CustomTypePromptAction::ValueInput(input_action) => {
                self.input.handle(input_action).into()
            }
        };

        Ok(result)
    }

    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        let prompt = &self.message;

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

        Ok(())
    }
}
