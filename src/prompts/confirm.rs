use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    answer::Answer,
    ask::{Question, QuestionOptions},
    config::{PromptConfig, Transformer, DEFAULT_TRANSFORMER},
    renderer::Renderer,
    Prompt,
};

const ERROR_MESSAGE: &str = "Invalid answer, try typing 'y' for yes or 'n' for no";

#[derive(Clone)]
pub struct ConfirmOptions<'a> {
    message: &'a str,
    default: Option<bool>,
    help_message: Option<&'a str>,
    transformer: Transformer,
}

impl<'a> ConfirmOptions<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            default: None,
            help_message: None,
            transformer: DEFAULT_TRANSFORMER,
        }
    }

    pub fn with_default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    pub fn with_transformer(mut self, transformer: Transformer) -> Self {
        self.transformer = transformer;
        self
    }
}

impl<'a> QuestionOptions<'a> for ConfirmOptions<'a> {
    fn with_config(mut self, global_config: &'a PromptConfig) -> Self {
        if let Some(transformer) = global_config.transformer {
            self.transformer = transformer;
        }
        if let Some(help_message) = global_config.help_message {
            self.help_message = Some(help_message);
        }
        if let Some(default) = global_config.confirm_default {
            self.default = Some(default);
        }

        self
    }

    fn into_question(self) -> Question<'a> {
        Question::Confirm(self)
    }
}

impl<'a> From<&'a str> for ConfirmOptions<'a> {
    fn from(val: &'a str) -> Self {
        ConfirmOptions::new(val)
    }
}

pub(in crate) struct Confirm<'a> {
    message: &'a str,
    error_state: bool,
    help_message: Option<&'a str>,
    default: Option<bool>,
    content: String,
    transformer: Transformer,
}

impl<'a> From<ConfirmOptions<'a>> for Confirm<'a> {
    fn from(co: ConfirmOptions<'a>) -> Self {
        Self {
            message: co.message,
            error_state: false,
            default: co.default,
            help_message: co.help_message,
            transformer: co.transformer,
            content: String::new(),
        }
    }
}

impl<'a> Confirm<'a> {
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

    fn get_final_answer(&self) -> Result<bool, ()> {
        lazy_static! {
            static ref YES_REGEX: Regex = Regex::new(r"^(?i:y(?:es)?)$").unwrap();
            static ref NO_REGEX: Regex = Regex::new(r"^(?i:n(?:o)?)$").unwrap();
        }

        match self.default {
            Some(val) if self.content.is_empty() => return Ok(val),
            _ => {}
        }

        if YES_REGEX.is_match(&self.content) {
            Ok(true)
        } else if NO_REGEX.is_match(&self.content) {
            Ok(false)
        } else {
            Err(())
        }
    }

    fn render(&mut self, renderer: &mut Renderer) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if self.error_state {
            renderer.print_error_message(ERROR_MESSAGE)?;
        }

        let default_message = self.default.map(|v| match v {
            true => "Y/n",
            false => "y/N",
        });

        renderer.print_prompt(&prompt, default_message, Some(&self.content))?;

        if let Some(message) = self.help_message {
            renderer.print_help(message)?;
        }

        renderer.flush()?;

        Ok(())
    }
}

impl<'a> Prompt for Confirm<'a> {
    fn prompt(mut self, renderer: &mut Renderer) -> Result<Answer, Box<dyn Error>> {
        let final_answer: Answer;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Ctrl('c') => bail!("Confirm interrupted by ctrl-c"),
                Key::Char('\n') | Key::Char('\r') => match self.get_final_answer() {
                    Ok(answer) => {
                        final_answer = Answer::Confirm(answer);
                        break;
                    }
                    Err(_) => {
                        self.error_state = true;
                        self.content.clear();
                    }
                },
                key => self.on_change(key),
            }
        }

        let transformed = (self.transformer)(&final_answer);

        renderer.cleanup(&self.message, &transformed)?;

        Ok(final_answer)
    }
}