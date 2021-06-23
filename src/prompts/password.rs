use lazy_static::__Deref;
use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    answer::{Answer, Prompt},
    ask::{Question, QuestionOptions},
    config::{PromptConfig, Transformer, Validator, DEFAULT_TRANSFORMER, DEFAULT_VALIDATOR},
    renderer::Renderer,
    terminal::Terminal,
};

#[derive(Copy, Clone)]
pub struct PasswordOptions<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    transformer: &'a Transformer,
    validator: Validator,
}

impl<'a> PasswordOptions<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            help_message: None,
            transformer: &DEFAULT_TRANSFORMER,
            validator: DEFAULT_VALIDATOR,
        }
    }

    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    pub fn with_transformer(mut self, transformer: &'a Transformer) -> Self {
        self.transformer = transformer;
        self
    }

    pub fn with_validator(mut self, validator: Validator) -> Self {
        self.validator = validator;
        self
    }
}

impl<'a> QuestionOptions<'a> for PasswordOptions<'a> {
    fn with_config(mut self, global_config: &'a PromptConfig) -> Self {
        if let Some(transformer) = global_config.transformer {
            self.transformer = transformer;
        }
        if let Some(help_message) = global_config.help_message {
            self.help_message = Some(help_message);
        }
        if let Some(validator) = global_config.validator {
            self.validator = validator;
        }

        self
    }

    fn into_question(self) -> Question<'a> {
        Question::Password(self)
    }
}

pub(in crate) struct Password<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    renderer: Renderer,
    content: String,
    transformer: &'a Transformer,
    validator: Validator,
    error: Option<Box<dyn Error>>,
}

impl<'a> From<PasswordOptions<'a>> for Password<'a> {
    fn from(so: PasswordOptions<'a>) -> Self {
        Self {
            message: so.message,
            help_message: so.help_message,
            renderer: Renderer::default(),
            transformer: so.transformer,
            validator: so.validator,
            content: String::new(),
            error: None,
        }
    }
}

impl<'a> From<&'a str> for PasswordOptions<'a> {
    fn from(val: &'a str) -> Self {
        PasswordOptions::new(val)
    }
}

impl<'a> Password<'a> {
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

    fn get_final_answer(&self) -> Result<Answer, Box<dyn Error>> {
        let answer = Answer::Password(self.content.clone());

        match (self.validator)(&answer) {
            Ok(_) => Ok(answer),
            Err(err) => Err(err),
        }
    }

    fn cleanup(&mut self, terminal: &mut Terminal, answer: &str) -> Result<(), Box<dyn Error>> {
        self.renderer.reset_prompt(terminal)?;
        self.renderer
            .print_prompt_answer(terminal, &self.message, answer)?;

        Ok(())
    }
}

impl<'a> Prompt for Password<'a> {
    fn render(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        self.renderer.reset_prompt(terminal)?;

        if let Some(err) = &self.error {
            self.renderer.print_error(terminal, err.deref())?;
        }

        self.renderer.print_prompt(terminal, &prompt, None, None)?;

        if let Some(message) = self.help_message {
            self.renderer.print_help(terminal, message)?;
        }

        terminal.flush()?;

        Ok(())
    }

    fn prompt(mut self) -> Result<Answer, Box<dyn Error>> {
        let mut terminal = Terminal::new()?;
        terminal.cursor_hide()?;

        let final_answer: Answer;

        loop {
            self.render(&mut terminal)?;

            let key = terminal.read_key()?;

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

        let transformed = (self.transformer)(&final_answer);

        self.cleanup(&mut terminal, &transformed)?;

        terminal.cursor_show()?;

        Ok(final_answer)
    }
}
