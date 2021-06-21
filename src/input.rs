use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    ask::{Question, QuestionOptions},
    config::{PromptConfig, Transformer, DEFAULT_TRANSFORMER},
    question::{Answer, Prompt},
    renderer::Renderer,
    terminal::Terminal,
};

#[derive(Copy, Clone)]
pub struct InputOptions<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    transformer: &'a Transformer,
}

impl<'a> InputOptions<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            help_message: None,
            transformer: &DEFAULT_TRANSFORMER,
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
}

impl<'a> QuestionOptions<'a> for InputOptions<'a> {
    fn with_config(mut self, global_config: &'a PromptConfig) -> Self {
        if let Some(transformer) = global_config.transformer {
            self.transformer = transformer;
        }
        if let Some(help_message) = global_config.help_message {
            self.help_message = Some(help_message);
        }

        self
    }

    fn into_question(self) -> Question<'a> {
        Question::Input(self)
    }
}

pub(in crate) struct Input<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    renderer: Renderer,
    content: String,
    transformer: &'a Transformer,
}

impl<'a> From<InputOptions<'a>> for Input<'a> {
    fn from(so: InputOptions<'a>) -> Self {
        Self {
            message: so.message,
            help_message: so.help_message,
            renderer: Renderer::default(),
            transformer: so.transformer,
            content: String::new(),
        }
    }
}

impl<'a> Input<'a> {
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
        Ok(Answer::Content(self.content.clone()))
    }

    fn cleanup(&mut self, terminal: &mut Terminal, answer: &str) -> Result<(), Box<dyn Error>> {
        self.renderer.reset_prompt(terminal)?;
        self.renderer
            .print_prompt_answer(terminal, &self.message, answer)?;

        Ok(())
    }
}

impl<'a> Prompt for Input<'a> {
    fn render(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        self.renderer.reset_prompt(terminal)?;

        self.renderer
            .print_prompt(terminal, &prompt, None, Some(&self.content))?;

        if let Some(message) = self.help_message {
            self.renderer.print_help(terminal, message)?;
        }

        terminal.flush()?;

        Ok(())
    }

    fn prompt(mut self) -> Result<Answer, Box<dyn Error>> {
        let mut terminal = Terminal::new()?;
        terminal.cursor_hide()?;

        loop {
            self.render(&mut terminal)?;

            let key = terminal.read_key()?;

            match key {
                Key::Ctrl('c') => bail!("Input interrupted by ctrl-c"),
                Key::Char('\n') | Key::Char('\r') => break,
                key => self.on_change(key),
            }
        }

        let answer = self.get_final_answer()?;
        let transformed = (self.transformer)(&answer);

        self.cleanup(&mut terminal, &transformed)?;

        terminal.cursor_show()?;

        Ok(answer)
    }
}
