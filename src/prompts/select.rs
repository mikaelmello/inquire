use lazy_static::__Deref;
use simple_error::SimpleError;
use std::{error::Error, iter::FromIterator};
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    config::{self, Filter},
    formatter::{self, OptionFormatter},
    renderer::Renderer,
    terminal::Terminal,
    utils::paginate,
    OptionAnswer,
};

#[derive(Copy, Clone)]
pub struct Select<'a> {
    pub message: &'a str,
    pub options: &'a [&'a str],
    pub help_message: Option<&'a str>,
    pub page_size: usize,
    pub vim_mode: bool,
    pub starting_cursor: usize,
    pub filter: Filter,
    pub formatter: OptionFormatter,
}

impl<'a> Select<'a> {
    pub const DEFAULT_FORMATTER: OptionFormatter = formatter::DEFAULT_OPTION_FORMATTER;
    pub const DEFAULT_FILTER: Filter = config::DEFAULT_FILTER;
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;
    pub const DEFAULT_VIM_MODE: bool = config::DEFAULT_VIM_MODE;
    pub const DEFAULT_STARTING_CURSOR: usize = 0;
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("↑↓ to move, space or enter to select, type to filter");

    pub fn new(message: &'a str, options: &'a [&str]) -> Self {
        Self {
            message,
            options,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            starting_cursor: Self::DEFAULT_STARTING_CURSOR,
            filter: Self::DEFAULT_FILTER,
            formatter: Self::DEFAULT_FORMATTER,
        }
    }

    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    pub fn without_help_message(mut self) -> Self {
        self.help_message = None;
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

    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_formatter(mut self, formatter: OptionFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Self {
        self.starting_cursor = starting_cursor;
        self
    }

    pub fn prompt(self) -> Result<OptionAnswer, Box<dyn Error>> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(
        self,
        renderer: &mut Renderer,
    ) -> Result<OptionAnswer, Box<dyn Error>> {
        SelectPrompt::new(self)?.prompt(renderer)
    }
}

pub(in crate) struct SelectPrompt<'a> {
    message: &'a str,
    options: &'a [&'a str],
    help_message: Option<&'a str>,
    vim_mode: bool,
    cursor_index: usize,
    page_size: usize,
    filter_value: Option<String>,
    filtered_options: Vec<usize>,
    filter: Filter,
    formatter: OptionFormatter,
    error: Option<Box<dyn Error>>,
}

impl<'a> SelectPrompt<'a> {
    fn new(so: Select<'a>) -> Result<Self, Box<dyn Error>> {
        if so.options.is_empty() {
            bail!("Please provide options to select from");
        }

        if so.starting_cursor >= so.options.len() {
            bail!("Starting selection should not be larger than length of options");
        }

        Ok(Self {
            message: so.message,
            options: so.options,
            help_message: so.help_message,
            vim_mode: so.vim_mode,
            cursor_index: so.starting_cursor,
            page_size: so.page_size,
            filter_value: None,
            filtered_options: Vec::from_iter(0..so.options.len()),
            filter: so.filter,
            formatter: so.formatter,
            error: None,
        })
    }

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
            .or(self.filtered_options.len().checked_sub(1))
            .unwrap_or_else(|| 0);
    }

    fn move_cursor_down(&mut self) {
        self.cursor_index = self.cursor_index.saturating_add(1);
        if self.cursor_index >= self.filtered_options.len() {
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

    fn get_final_answer(&self) -> Result<OptionAnswer, Box<dyn Error>> {
        self.filtered_options
            .get(self.cursor_index)
            .and_then(|i| self.options.get(*i).map(|opt| OptionAnswer::new(*i, opt)))
            .ok_or(Box::new(SimpleError::new("Invalid selected index")))
    }

    fn render(&mut self, renderer: &mut Renderer) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(err) = &self.error {
            renderer.print_error(err.deref())?;
        }

        renderer.print_prompt(&prompt, None, self.filter_value.as_deref())?;

        let choices = self
            .filtered_options
            .iter()
            .cloned()
            .map(|i| OptionAnswer::new(i, self.options.get(i).unwrap()))
            .collect::<Vec<OptionAnswer>>();

        let (paginated_opts, rel_sel) = paginate(self.page_size, &choices, self.cursor_index);

        for (idx, opt) in paginated_opts.iter().enumerate() {
            renderer.print_option(rel_sel == idx, &opt.value)?;
        }

        if let Some(help_message) = self.help_message {
            renderer.print_help(help_message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> Result<OptionAnswer, Box<dyn Error>> {
        let final_answer: OptionAnswer;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Ctrl('c') => bail!("Multi-selection interrupted by ctrl-c"),
                Key::Char('\n') | Key::Char('\r') | Key::Char(' ') => match self.get_final_answer()
                {
                    Ok(answer) => {
                        final_answer = answer;
                        break;
                    }
                    Err(err) => self.error = Some(err),
                },
                key => self.on_change(key),
            }
        }

        let transformed = (self.formatter)(&final_answer);

        renderer.cleanup(&self.message, &transformed)?;

        Ok(final_answer)
    }
}
