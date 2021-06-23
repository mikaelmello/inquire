use lazy_static::__Deref;
use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    answer::{Answer, Prompt},
    ask::{Question, QuestionOptions},
    config::{
        PromptConfig, Suggestor, Transformer, Validator, DEFAULT_PAGE_SIZE, DEFAULT_TRANSFORMER,
        DEFAULT_VALIDATOR,
    },
    renderer::Renderer,
    terminal::Terminal,
    utils::paginate,
    OptionAnswer,
};

const DEFAULT_HELP_MESSAGE: &str = "↑↓ to move, tab to auto-complete, enter to submit";

#[derive(Copy, Clone)]
pub struct InputOptions<'a> {
    message: &'a str,
    default: Option<&'a str>,
    help_message: Option<&'a str>,
    transformer: Transformer,
    validator: Validator,
    page_size: usize,
    suggestor: Option<Suggestor>,
}

impl<'a> InputOptions<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            default: None,
            help_message: None,
            transformer: DEFAULT_TRANSFORMER,
            validator: DEFAULT_VALIDATOR,
            page_size: DEFAULT_PAGE_SIZE,
            suggestor: None,
        }
    }

    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    pub fn with_default(mut self, message: &'a str) -> Self {
        self.default = Some(message);
        self
    }

    pub fn with_suggestor(mut self, suggestor: Suggestor) -> Self {
        self.suggestor = Some(suggestor);
        self
    }

    pub fn with_transformer(mut self, transformer: Transformer) -> Self {
        self.transformer = transformer;
        self
    }

    pub fn with_validator(mut self, validator: Validator) -> Self {
        self.validator = validator;
        self
    }
}

impl<'a> QuestionOptions<'a> for InputOptions<'a> {
    fn with_config(mut self, global_config: &'a PromptConfig) -> Self {
        if let Some(transformer) = global_config.transformer {
            self.transformer = transformer;
        }
        if let Some(validator) = global_config.validator {
            self.validator = validator;
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
    default: Option<&'a str>,
    help_message: Option<&'a str>,
    renderer: Renderer,
    content: String,
    transformer: Transformer,
    validator: Validator,
    error: Option<Box<dyn Error>>,
    suggestor: Option<Suggestor>,
    suggested_options: Vec<String>,
    cursor_index: usize,
    page_size: usize,
}

impl<'a> From<InputOptions<'a>> for Input<'a> {
    fn from(so: InputOptions<'a>) -> Self {
        Self {
            message: so.message,
            default: so.default,
            help_message: so.help_message,
            renderer: Renderer::default(),
            transformer: so.transformer,
            validator: so.validator,
            suggestor: so.suggestor,
            content: String::new(),
            error: None,
            cursor_index: 0,
            page_size: so.page_size,
            suggested_options: match so.suggestor {
                Some(s) => s(""),
                None => vec![],
            },
        }
    }
}

impl<'a> From<&'a str> for InputOptions<'a> {
    fn from(val: &'a str) -> Self {
        InputOptions::new(val)
    }
}

impl<'a> Input<'a> {
    fn update_suggestions(&mut self) {
        match self.suggestor {
            Some(suggestor) => {
                self.suggested_options = suggestor(&self.content);
                if self.suggested_options.len() > 0
                    && self.suggested_options.len() <= self.cursor_index
                {
                    self.cursor_index = self.suggested_options.len().saturating_sub(1);
                }
            }
            _ => {}
        }
    }

    fn move_cursor_up(&mut self) {
        self.cursor_index = self
            .cursor_index
            .checked_sub(1)
            .or(self.suggested_options.len().checked_sub(1))
            .unwrap_or_else(|| 0);
    }

    fn move_cursor_down(&mut self) {
        self.cursor_index = self.cursor_index.saturating_add(1);
        if self.cursor_index >= self.suggested_options.len() {
            self.cursor_index = 0;
        }
    }

    fn on_change(&mut self, key: Key) {
        let mut dirty = false;

        match key {
            Key::Backspace => {
                let len = self.content[..].graphemes(true).count();
                let new_len = len.saturating_sub(1);
                self.content = self.content[..].graphemes(true).take(new_len).collect();
                dirty = true;
            }
            Key::Up => self.move_cursor_up(),
            Key::Down => self.move_cursor_down(),
            Key::Char('\x17') | Key::Char('\x18') => {
                self.content.clear();
                dirty = true;
            }
            Key::Char(c) => {
                self.content.push(c);
                dirty = true;
            }
            _ => {}
        }

        if dirty {
            self.update_suggestions();
        }
    }

    fn use_select_option(&mut self) {
        let selected_suggestion = self.suggested_options.get(self.cursor_index);

        if let Some(ans) = selected_suggestion {
            self.content = ans.clone();
            self.update_suggestions();
        }
    }

    fn get_final_answer(&self) -> Result<Answer, Box<dyn Error>> {
        match self.default {
            Some(val) if self.content.is_empty() => return Ok(Answer::Content(val.to_string())),
            _ => {}
        }

        let answer = Answer::Content(self.content.clone());

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

impl<'a> Prompt for Input<'a> {
    fn render(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        self.renderer.reset_prompt(terminal)?;

        if let Some(err) = &self.error {
            self.renderer.print_error(terminal, err.deref())?;
        }

        self.renderer
            .print_prompt(terminal, &prompt, self.default, Some(&self.content))?;

        let choices = self
            .suggested_options
            .iter()
            .enumerate()
            .map(|(i, val)| OptionAnswer::new(i, val))
            .collect::<Vec<OptionAnswer>>();

        let (paginated_opts, rel_sel) = paginate(self.page_size, &choices, self.cursor_index);
        for (idx, opt) in paginated_opts.iter().enumerate() {
            self.renderer
                .print_option(terminal, rel_sel == idx, &opt.value)?;
        }

        if let Some(message) = self.help_message {
            self.renderer.print_help(terminal, message)?;
        } else if !choices.is_empty() {
            self.renderer.print_help(terminal, DEFAULT_HELP_MESSAGE)?;
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
                Key::Ctrl('c') => bail!("Input interrupted by ctrl-c"),
                Key::Char('\t') => self.use_select_option(),
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
