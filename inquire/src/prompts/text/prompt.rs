use std::cmp::min;

use crate::{
    autocompletion::Replacement,
    error::InquireResult,
    formatter::StringFormatter,
    input::{Input, InputActionResult},
    list_option::ListOption,
    prompts::prompt::{ActionResult, Prompt},
    ui::TextBackend,
    utils::paginate,
    validator::{ErrorMessage, StringValidator, Validation},
    Autocomplete, InquireError, Text,
};

use super::{action::TextPromptAction, config::TextConfig};

pub struct TextPrompt<'a> {
    message: &'a str,
    config: TextConfig,
    default: Option<&'a str>,
    help_message: Option<String>,
    input: Input,
    formatter: StringFormatter<'a>,
    validators: Vec<Box<dyn StringValidator>>,
    error: Option<ErrorMessage>,
    autocompleter: Option<Box<dyn Autocomplete>>,
    suggested_options: Vec<String>,
    suggestion_cursor_index: Option<usize>,
}

impl<'a> From<Text<'a>> for TextPrompt<'a> {
    fn from(so: Text<'a>) -> Self {
        let config = (&so).into();
        let input = Input::new_with(so.initial_value.unwrap_or_default());
        let input = if let Some(placeholder) = so.placeholder {
            input.with_placeholder(placeholder)
        } else {
            input
        };

        let default_help_message = if so.autocompleter.is_some() {
            Some("↑↓ to move, tab to autocomplete, enter to submit")
        } else {
            None
        };
        let help_message = so
            .help_message
            .into_or_default(default_help_message.map(|s| s.into()));

        Self {
            message: so.message,
            config,
            default: so.default,
            help_message,
            formatter: so.formatter,
            autocompleter: so.autocompleter,
            input,
            error: None,
            suggestion_cursor_index: None,
            suggested_options: vec![],
            validators: so.validators,
        }
    }
}

impl<'a> TextPrompt<'a> {
    fn update_suggestions(&mut self) -> InquireResult<()> {
        if let Some(autocompleter) = &mut self.autocompleter {
            self.suggested_options = autocompleter.get_suggestions(self.input.content())?;
            self.suggestion_cursor_index = None;
        }

        Ok(())
    }

    fn get_highlighted_suggestion(&self) -> Option<&str> {
        if let Some(cursor) = self.suggestion_cursor_index {
            let suggestion = self.suggested_options.get(cursor).unwrap().as_ref();
            Some(suggestion)
        } else {
            None
        }
    }

    fn move_cursor_up(&mut self, qty: usize) -> ActionResult {
        let new_cursor_index = match self.suggestion_cursor_index {
            None => None,
            Some(index) if index < qty => None,
            Some(index) => Some(index.saturating_sub(qty)),
        };

        self.update_suggestion_cursor_pos(new_cursor_index)
    }

    fn move_cursor_down(&mut self, qty: usize) -> ActionResult {
        let new_cursor_index = match self.suggested_options.is_empty() {
            true => None,
            false => match self.suggestion_cursor_index {
                None if qty == 0 => None,
                None => Some(min(
                    qty.saturating_sub(1),
                    self.suggested_options.len().saturating_sub(1),
                )),
                Some(index) => Some(min(
                    index.saturating_add(qty),
                    self.suggested_options.len().saturating_sub(1),
                )),
            },
        };

        self.update_suggestion_cursor_pos(new_cursor_index)
    }

    fn update_suggestion_cursor_pos(&mut self, new_position: Option<usize>) -> ActionResult {
        if new_position != self.suggestion_cursor_index {
            self.suggestion_cursor_index = new_position;
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    fn use_current_suggestion(&mut self) -> InquireResult<ActionResult> {
        let suggestion = self.get_highlighted_suggestion().map(|s| s.to_owned());
        if let Some(autocompleter) = &mut self.autocompleter {
            match autocompleter.get_completion(self.input.content(), suggestion)? {
                Replacement::Some(value) => {
                    self.input = Input::new_with(value);
                    Ok(ActionResult::NeedsRedraw)
                }
                Replacement::None => Ok(ActionResult::Clean),
            }
        } else {
            Ok(ActionResult::Clean)
        }
    }

    fn get_current_answer(&self) -> &str {
        // If there is a highlighted suggestion, assume user wanted it as
        // the answer.
        if let Some(suggestion) = self.get_highlighted_suggestion() {
            return suggestion;
        }

        // Empty input with default values override any validators.
        if self.input.content().is_empty() {
            if let Some(val) = self.default {
                return val;
            }
        }

        self.input.content()
    }

    fn validate_current_answer(&self) -> InquireResult<Validation> {
        for validator in &self.validators {
            match validator.validate(self.get_current_answer()) {
                Ok(Validation::Valid) => {}
                Ok(Validation::Invalid(msg)) => return Ok(Validation::Invalid(msg)),
                Err(err) => return Err(InquireError::Custom(err)),
            }
        }

        Ok(Validation::Valid)
    }
}

impl<'a, B> Prompt<'a, B, TextConfig, TextPromptAction, String> for TextPrompt<'a>
where
    B: TextBackend,
{
    fn message(&self) -> &str {
        self.message
    }

    fn help_message(&self) -> Option<&str> {
        self.help_message.as_deref()
    }

    fn config(&self) -> &TextConfig {
        &self.config
    }

    fn format_answer(&self, answer: &String) -> String {
        (self.formatter)(answer)
    }

    fn setup(&mut self) -> InquireResult<()> {
        self.update_suggestions()
    }

    fn submit(&mut self) -> InquireResult<Option<String>> {
        let result = match self.validate_current_answer()? {
            Validation::Valid => Some(self.get_current_answer().to_owned()),
            Validation::Invalid(msg) => {
                self.error = Some(msg);
                None
            }
        };

        Ok(result)
    }

    fn handle(&mut self, action: TextPromptAction) -> InquireResult<ActionResult> {
        let result = match action {
            TextPromptAction::ValueInput(input_action) => {
                let result = self.input.handle(input_action);

                if let InputActionResult::ContentChanged = result {
                    self.update_suggestions()?;
                }

                result.into()
            }
            TextPromptAction::MoveToSuggestionAbove => self.move_cursor_up(1),
            TextPromptAction::MoveToSuggestionBelow => self.move_cursor_down(1),
            TextPromptAction::MoveToSuggestionPageUp => self.move_cursor_up(self.config.page_size),
            TextPromptAction::MoveToSuggestionPageDown => {
                self.move_cursor_down(self.config.page_size)
            }
            TextPromptAction::UseCurrentSuggestion => self.use_current_suggestion()?,
        };

        Ok(result)
    }

    fn render(&self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        backend.render_prompt(prompt, self.default, &self.input)?;

        let choices = self
            .suggested_options
            .iter()
            .enumerate()
            .map(|(i, val)| ListOption::new(i, val.as_ref()))
            .collect::<Vec<ListOption<&str>>>();

        let page = paginate(
            self.config.page_size,
            &choices,
            self.suggestion_cursor_index,
        );

        backend.render_suggestions(page)?;

        Ok(())
    }
}
