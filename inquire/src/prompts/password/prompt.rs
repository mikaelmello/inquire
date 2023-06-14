use crate::{
    error::InquireResult,
    formatter::StringFormatter,
    input::Input,
    prompts::prompt::{ActionResult, Prompt},
    ui::PasswordBackend,
    validator::{ErrorMessage, StringValidator, Validation},
    InquireError, Password, PasswordDisplayMode,
};

use super::{action::PasswordPromptAction, config::PasswordConfig};

// Helper type for representing the password confirmation flow.
struct PasswordConfirmation<'a> {
    // The message of the prompt.
    message: &'a str,

    // The error message of the prompt.
    error_message: &'a str,

    // The input to confirm.
    input: Input,
}

pub struct PasswordPrompt<'a> {
    message: &'a str,
    config: PasswordConfig,
    help_message: Option<&'a str>,
    input: Input,
    current_mode: PasswordDisplayMode,
    confirmation: Option<PasswordConfirmation<'a>>, // if `None`, confirmation is disabled, `Some(_)` confirmation is enabled
    confirmation_stage: bool,
    formatter: StringFormatter<'a>,
    validators: Vec<Box<dyn StringValidator>>,
    error: Option<ErrorMessage>,
}

impl<'a> From<Password<'a>> for PasswordPrompt<'a> {
    fn from(so: Password<'a>) -> Self {
        let confirmation = match so.enable_confirmation {
            true => Some(PasswordConfirmation {
                message: so.custom_confirmation_message.unwrap_or("Confirmation:"),
                error_message: so
                    .custom_confirmation_error_message
                    .unwrap_or("The answers don't match."),
                input: Input::new(),
            }),
            false => None,
        };

        Self {
            message: so.message,
            config: (&so).into(),
            help_message: so.help_message,
            current_mode: so.display_mode,
            confirmation,
            confirmation_stage: false,
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
    fn active_input(&self) -> &Input {
        match &self.confirmation {
            Some(confirmation) if self.confirmation_stage => &confirmation.input,
            _ => &self.input,
        }
    }

    fn active_input_mut(&mut self) -> &mut Input {
        match &mut self.confirmation {
            Some(confirmation) if self.confirmation_stage => &mut confirmation.input,
            _ => &mut self.input,
        }
    }

    fn toggle_display_mode(&mut self) -> ActionResult {
        let new_mode = match self.current_mode {
            PasswordDisplayMode::Hidden | PasswordDisplayMode::Masked => PasswordDisplayMode::Full,
            PasswordDisplayMode::Full => self.config.display_mode,
        };

        if new_mode != self.current_mode {
            self.current_mode = new_mode;
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    fn confirm_current_answer(&mut self) -> Option<String> {
        let cur_answer = self.cur_answer();
        match &mut self.confirmation {
            None => Some(cur_answer),
            Some(confirmation) => {
                if !self.confirmation_stage {
                    if self.current_mode == PasswordDisplayMode::Hidden {
                        confirmation.input.clear();
                    }

                    self.error = None;
                    self.confirmation_stage = true;

                    None
                } else if self.input.content() == cur_answer {
                    Some(confirmation.input.content().into())
                } else {
                    confirmation.input.clear();

                    self.error = Some(confirmation.error_message.into());
                    self.confirmation_stage = false;

                    None
                }
            }
        }
    }

    fn validate_current_answer(&self) -> InquireResult<Validation> {
        for validator in &self.validators {
            match validator.validate(self.active_input().content()) {
                Ok(Validation::Valid) => {}
                Ok(Validation::Invalid(msg)) => return Ok(Validation::Invalid(msg)),
                Err(err) => return Err(InquireError::Custom(err)),
            }
        }

        Ok(Validation::Valid)
    }

    fn cur_answer(&self) -> String {
        self.active_input().content().into()
    }
}

impl<'a, B> Prompt<B, PasswordConfig, PasswordPromptAction, String> for PasswordPrompt<'a>
where
    B: PasswordBackend,
{
    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &PasswordConfig {
        &self.config
    }

    fn format_answer(&self, answer: &String) -> String {
        (self.formatter)(answer)
    }

    fn pre_cancel(&mut self) -> InquireResult<bool> {
        if self.confirmation_stage && self.confirmation.is_some() {
            if self.current_mode == PasswordDisplayMode::Hidden {
                self.input.clear();
            }

            self.error = None;
            self.confirmation_stage = false;

            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn submit(&mut self) -> InquireResult<Option<String>> {
        let answer = match self.validate_current_answer()? {
            Validation::Valid => self.confirm_current_answer(),
            Validation::Invalid(msg) => {
                self.error = Some(msg);
                None
            }
        };

        Ok(answer)
    }

    fn handle(&mut self, action: PasswordPromptAction) -> InquireResult<ActionResult> {
        let result = match action {
            PasswordPromptAction::ValueInput(input_action) => {
                self.active_input_mut().handle(input_action).into()
            }
            PasswordPromptAction::ToggleDisplayMode => self.toggle_display_mode(),
        };

        Ok(result)
    }

    fn render(&self, backend: &mut B) -> InquireResult<()> {
        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        match self.current_mode {
            PasswordDisplayMode::Hidden => {
                backend.render_prompt(self.message)?;

                match &self.confirmation {
                    Some(confirmation) if self.confirmation_stage => {
                        backend.render_prompt(confirmation.message)?;
                    }
                    _ => {}
                }
            }
            PasswordDisplayMode::Masked => {
                backend.render_prompt_with_masked_input(self.message, &self.input)?;

                match &self.confirmation {
                    Some(confirmation) if self.confirmation_stage => {
                        backend.render_prompt_with_masked_input(
                            confirmation.message,
                            &confirmation.input,
                        )?;
                    }
                    _ => {}
                }
            }
            PasswordDisplayMode::Full => {
                backend.render_prompt_with_full_input(self.message, &self.input)?;

                match &self.confirmation {
                    Some(confirmation) if self.confirmation_stage => {
                        backend.render_prompt_with_full_input(
                            confirmation.message,
                            &confirmation.input,
                        )?;
                    }
                    _ => {}
                }
            }
        }

        if let Some(message) = self.help_message {
            backend.render_help_message(message)?;
        }

        Ok(())
    }
}
