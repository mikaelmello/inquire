use std::{collections::HashSet, error::Error, iter::FromIterator};
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    config::{self, Filter},
    formatter::{self, MultiOptionFormatter},
    renderer::Renderer,
    terminal::Terminal,
    utils::paginate,
    validator::MultiOptionValidator,
    OptionAnswer,
};

#[derive(Copy, Clone)]
pub struct MultiSelect<'a> {
    pub message: &'a str,
    pub options: &'a [&'a str],
    pub default: Option<&'a [usize]>,
    pub help_message: Option<&'a str>,
    pub page_size: usize,
    pub vim_mode: bool,
    pub starting_cursor: usize,
    pub filter: Filter,
    pub keep_filter: bool,
    pub formatter: MultiOptionFormatter,
    pub validator: Option<MultiOptionValidator>,
}

impl<'a> MultiSelect<'a> {
    pub const DEFAULT_FORMATTER: MultiOptionFormatter = formatter::DEFAULT_MULTI_OPTION_FORMATTER;
    pub const DEFAULT_FILTER: Filter = config::DEFAULT_FILTER;
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;
    pub const DEFAULT_VIM_MODE: bool = config::DEFAULT_VIM_MODE;
    pub const DEFAULT_KEEP_FILTER: bool = true;
    pub const DEFAULT_STARTING_CURSOR: usize = 0;
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("↑↓ to move, space to select one, → to all, ← to none, type to filter");

    pub fn new(message: &'a str, options: &'a [&str]) -> Self {
        Self {
            message,
            options,
            default: None,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            starting_cursor: Self::DEFAULT_STARTING_CURSOR,
            keep_filter: Self::DEFAULT_KEEP_FILTER,
            filter: Self::DEFAULT_FILTER,
            formatter: Self::DEFAULT_FORMATTER,
            validator: None,
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

    pub fn with_keep_filter(mut self, keep_filter: bool) -> Self {
        self.keep_filter = keep_filter;
        self
    }

    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_formatter(mut self, formatter: MultiOptionFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn with_validator(mut self, validator: MultiOptionValidator) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn with_default(mut self, default: &'a [usize]) -> Self {
        self.default = Some(default);
        self
    }

    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Self {
        self.starting_cursor = starting_cursor;
        self
    }

    pub fn prompt(self) -> Result<Vec<OptionAnswer>, Box<dyn Error>> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(
        self,
        renderer: &mut Renderer,
    ) -> Result<Vec<OptionAnswer>, Box<dyn Error>> {
        MultiSelectPrompt::new(self)?.prompt(renderer)
    }
}

struct MultiSelectPrompt<'a> {
    message: &'a str,
    options: &'a [&'a str],
    help_message: Option<&'a str>,
    vim_mode: bool,
    cursor_index: usize,
    checked: HashSet<usize>,
    page_size: usize,
    keep_filter: bool,
    filter_value: Option<String>,
    filtered_options: Vec<usize>,
    filter: Filter,
    formatter: MultiOptionFormatter,
    validator: Option<MultiOptionValidator>,
    error: Option<String>,
}

impl<'a> MultiSelectPrompt<'a> {
    fn new(mso: MultiSelect<'a>) -> Result<Self, Box<dyn Error>> {
        if mso.options.is_empty() {
            bail!("Please provide options to select from");
        }
        if let Some(default) = mso.default {
            for i in default {
                if i >= &mso.options.len() {
                    bail!("Invalid index, larger than options available");
                }
            }
        }

        Ok(Self {
            message: mso.message,
            options: mso.options,
            help_message: mso.help_message,
            vim_mode: mso.vim_mode,
            cursor_index: mso.starting_cursor,
            page_size: mso.page_size,
            keep_filter: mso.keep_filter,
            filter_value: None,
            filtered_options: Vec::from_iter(0..mso.options.len()),
            filter: mso.filter,
            formatter: mso.formatter,
            validator: mso.validator,
            error: None,
            checked: mso
                .default
                .map_or_else(|| HashSet::new(), |d| d.iter().cloned().collect()),
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

    fn toggle_cursor_selection(&mut self) {
        let idx = match self.filtered_options.get(self.cursor_index) {
            Some(val) => val,
            None => return,
        };

        if self.checked.contains(idx) {
            self.checked.remove(idx);
        } else {
            self.checked.insert(*idx);
        }

        if !self.keep_filter {
            self.filter_value = None;
        }
    }

    fn on_change(&mut self, key: Key) {
        let old_filter = self.filter_value.clone();

        match key {
            Key::Up => self.move_cursor_up(),
            Key::Char('k') if self.vim_mode => self.move_cursor_up(),
            Key::Char('\t') | Key::Down => self.move_cursor_down(),
            Key::Char('j') if self.vim_mode => self.move_cursor_down(),
            Key::Char(' ') => self.toggle_cursor_selection(),
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
            Key::Right => {
                self.checked.clear();
                for idx in &self.filtered_options {
                    self.checked.insert(*idx);
                }

                if !self.keep_filter {
                    self.filter_value = None;
                }
            }
            Key::Left => {
                self.checked.clear();

                if !self.keep_filter {
                    self.filter_value = None;
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

    fn get_final_answer(&self) -> Result<Vec<OptionAnswer>, String> {
        let selected_options = self
            .options
            .iter()
            .enumerate()
            .filter_map(|(idx, opt)| match &self.checked.contains(&idx) {
                true => Some(OptionAnswer::new(idx, opt)),
                false => None,
            })
            .collect::<Vec<OptionAnswer>>();

        if let Some(validator) = self.validator {
            return match validator(&selected_options) {
                Ok(_) => Ok(selected_options),
                Err(err) => Err(err.to_string()),
            };
        }

        return Ok(selected_options);
    }

    fn render(&mut self, renderer: &mut Renderer) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(err) = &self.error {
            renderer.print_error_message(err)?;
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
            renderer.print_multi_option(
                rel_sel == idx,
                self.checked.contains(&opt.index),
                &opt.value,
            )?;
        }

        if let Some(help_message) = self.help_message {
            renderer.print_help(help_message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> Result<Vec<OptionAnswer>, Box<dyn Error>> {
        let final_answer: Vec<OptionAnswer>;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Ctrl('c') => bail!("Multi-selection interrupted by ctrl-c"),
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

        let formatted = (self.formatter)(&final_answer);

        renderer.cleanup(&self.message, &formatted)?;

        Ok(final_answer)
    }
}
