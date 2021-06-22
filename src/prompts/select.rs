use simple_error::SimpleError;
use std::{error::Error, iter::FromIterator};
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    answer::{Answer, Prompt},
    ask::{Question, QuestionOptions},
    config::{
        Filter, PromptConfig, Transformer, DEFAULT_FILTER, DEFAULT_PAGE_SIZE, DEFAULT_TRANSFORMER,
        DEFAULT_VIM_MODE,
    },
    renderer::Renderer,
    survey::OptionAnswer,
    terminal::Terminal,
    utils::paginate,
};

const DEFAULT_STARTING_SELECTION: usize = 0;
const DEFAULT_HELP_MESSAGE: &str = "↑↓ to move, space or enter to select, type to filter";

#[derive(Copy, Clone)]
pub struct SelectOptions<'a> {
    message: &'a str,
    options: &'a [&'a str],
    help_message: &'a str,
    page_size: usize,
    vim_mode: bool,
    starting_selection: usize,
    filter: &'a Filter,
    transformer: &'a Transformer,
}

impl<'a> SelectOptions<'a> {
    pub fn new(message: &'a str, options: &'a [&str]) -> Result<Self, Box<dyn Error>> {
        if options.is_empty() {
            bail!("Please provide options to select from");
        }

        Ok(Self {
            message,
            options,
            help_message: DEFAULT_HELP_MESSAGE,
            page_size: DEFAULT_PAGE_SIZE,
            vim_mode: DEFAULT_VIM_MODE,
            starting_selection: DEFAULT_STARTING_SELECTION,
            filter: &DEFAULT_FILTER,
            transformer: &DEFAULT_TRANSFORMER,
        })
    }

    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = message;
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    pub fn with_filter(mut self, filter: &'a Filter) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_transformer(mut self, transformer: &'a Transformer) -> Self {
        self.transformer = transformer;
        self
    }

    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Result<Self, Box<dyn Error>> {
        if starting_cursor >= self.options.len() {
            bail!("Starting selection should not be larger than length of options");
        }

        self.starting_selection = starting_cursor;
        Ok(self)
    }
}

impl<'a> QuestionOptions<'a> for SelectOptions<'a> {
    fn with_config(mut self, global_config: &'a PromptConfig) -> Self {
        if let Some(page_size) = global_config.page_size {
            self.page_size = page_size;
        }
        if let Some(vim_mode) = global_config.vim_mode {
            self.vim_mode = vim_mode;
        }
        if let Some(filter) = global_config.filter {
            self.filter = filter;
        }
        if let Some(transformer) = global_config.transformer {
            self.transformer = transformer;
        }
        if let Some(help_message) = global_config.help_message {
            self.help_message = help_message;
        }

        self
    }

    fn into_question(self) -> Question<'a> {
        Question::Select(self)
    }
}

pub(in crate) struct Select<'a> {
    message: &'a str,
    options: &'a [&'a str],
    help_message: &'a str,
    vim_mode: bool,
    cursor_index: usize,
    page_size: usize,
    renderer: Renderer,
    filter_value: Option<String>,
    filtered_options: Vec<usize>,
    filter: &'a Filter,
    transformer: &'a Transformer,
}

impl<'a> From<SelectOptions<'a>> for Select<'a> {
    fn from(so: SelectOptions<'a>) -> Self {
        Self {
            message: so.message,
            options: so.options,
            help_message: so.help_message,
            vim_mode: so.vim_mode,
            cursor_index: so.starting_selection,
            renderer: Renderer::default(),
            page_size: so.page_size,
            filter_value: None,
            filtered_options: Vec::from_iter(0..so.options.len()),
            filter: so.filter,
            transformer: so.transformer,
        }
    }
}

impl<'a> Select<'a> {
    fn filter_options(&self) -> Vec<usize> {
        self.options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| match &self.filter_value {
                Some(val) if (self.filter)(&val, opt, i) => Some(i),
                Some(_) => None,
                None => Some(i),
            })
            .collect()
    }

    fn move_cursor_up(&mut self) {
        self.cursor_index = self
            .cursor_index
            .checked_sub(1)
            .unwrap_or_else(|| self.filtered_options.len() - 1);
    }

    fn move_cursor_down(&mut self) {
        self.cursor_index = self.cursor_index.saturating_add(1);
        if self.cursor_index == self.filtered_options.len() {
            self.cursor_index = 0;
        }
    }

    fn on_change(&mut self, key: Key) {
        let old_filter = self.filter_value.clone();

        match key {
            Key::Up => self.move_cursor_up(),
            Key::Char('k') if self.vim_mode => self.move_cursor_up(),
            Key::Char('\t') | Key::Down => self.move_cursor_down(),
            Key::Char('j') if self.vim_mode => self.move_cursor_down(),
            Key::Char('\x17') | Key::Char('\x18') => {
                self.filter_value = None;
            }
            Key::Backspace => {
                if let Some(filter) = &self.filter_value {
                    let len = filter[..].graphemes(true).count();
                    let new_len = len.saturating_sub(1);
                    self.filter_value = Some(filter[..].graphemes(true).take(new_len).collect());
                }
            }
            Key::Char(c) => match &mut self.filter_value {
                Some(val) => val.push(c),
                None => self.filter_value = Some(String::from(c)),
            },
            _ => {}
        }

        if self.filter_value != old_filter {
            let options = self.filter_options();
            if options.len() > 0 && options.len() <= self.cursor_index {
                self.cursor_index = options.len().saturating_sub(1);
            }
            self.filtered_options = options;
        }
    }

    fn get_final_answer(&self) -> Result<Answer, Box<dyn Error>> {
        self.filtered_options
            .get(self.cursor_index)
            .and_then(|i| self.options.get(*i).map(|opt| OptionAnswer::new(*i, opt)))
            .map(|o| Answer::Option(o))
            .ok_or(Box::new(SimpleError::new("Invalid selected index")))
    }

    fn cleanup(&mut self, terminal: &mut Terminal, answer: &str) -> Result<(), Box<dyn Error>> {
        self.renderer.reset_prompt(terminal)?;
        self.renderer
            .print_prompt_answer(terminal, &self.message, answer)?;

        Ok(())
    }
}

impl<'a> Prompt for Select<'a> {
    fn render(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        self.renderer.reset_prompt(terminal)?;

        self.renderer
            .print_prompt(terminal, &prompt, None, self.filter_value.as_deref())?;

        let choices = self
            .filtered_options
            .iter()
            .cloned()
            .map(|i| OptionAnswer::new(i, self.options.get(i).unwrap()))
            .collect::<Vec<OptionAnswer>>();

        let (paginated_opts, rel_sel) = paginate(self.page_size, &choices, self.cursor_index);

        for (idx, opt) in paginated_opts.iter().enumerate() {
            self.renderer
                .print_option(terminal, rel_sel == idx, &opt.value)?;
        }

        self.renderer.print_help(terminal, self.help_message)?;

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
                Key::Ctrl('c') => bail!("Multi-selection interrupted by ctrl-c"),
                Key::Char('\n') | Key::Char('\r') | Key::Char(' ') => break,
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
