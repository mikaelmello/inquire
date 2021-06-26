use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    formatter::{BoolFormatter, DEFAULT_BOOL_FORMATTER},
    renderer::Renderer,
    terminal::Terminal,
};

const ERROR_MESSAGE: &str = "Invalid answer, try typing 'y' for yes or 'n' for no";

#[derive(Copy, Clone)]
pub struct Confirm<'a> {
    pub message: &'a str,
    pub default: Option<bool>,
    pub help_message: Option<&'a str>,
    pub formatter: BoolFormatter<'a>,
}

impl<'a> Confirm<'a> {
    pub const DEFAULT_FORMATTER: BoolFormatter<'a> = DEFAULT_BOOL_FORMATTER;

    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            default: None,
            help_message: None,
            formatter: Self::DEFAULT_FORMATTER,
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

    pub fn with_formatter(mut self, formatter: BoolFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn prompt(self) -> Result<bool, Box<dyn Error>> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(
        self,
        renderer: &mut Renderer,
    ) -> Result<bool, Box<dyn Error>> {
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
    error_state: bool,
    help_message: Option<&'a str>,
    default: Option<bool>,
    content: String,
    formatter: BoolFormatter<'a>,
}

impl<'a> From<Confirm<'a>> for ConfirmPrompt<'a> {
    fn from(co: Confirm<'a>) -> Self {
        Self {
            message: co.message,
            error_state: false,
            default: co.default,
            help_message: co.help_message,
            formatter: co.formatter,
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

    fn prompt(mut self, renderer: &mut Renderer) -> Result<bool, Box<dyn Error>> {
        let final_answer: bool;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Ctrl('c') => bail!("Confirm interrupted by ctrl-c"),
                Key::Char('\n') | Key::Char('\r') => match self.get_final_answer() {
                    Ok(answer) => {
                        final_answer = answer;
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

        let transformed = (self.formatter)(final_answer);

        renderer.cleanup(&self.message, &transformed)?;

        Ok(final_answer)
    }
}
