use std::{collections::HashSet, error::Error, iter::FromIterator};
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    ask::QuestionOptions,
    config::{
        Filter, PromptConfig, Transformer, DEFAULT_FILTER, DEFAULT_KEEP_FILTER, DEFAULT_PAGE_SIZE,
        DEFAULT_TRANSFORMER, DEFAULT_VIM_MODE,
    },
    question::{Answer, Prompt},
    renderer::Renderer,
    survey::OptionAnswer,
    terminal::Terminal,
    utils::paginate,
};

const DEFAULT_STARTING_SELECTION: usize = 0;
const DEFAULT_HELP_MESSAGE: &str =
    "↑↓ to move, space to select one, → to all, ← to none, type to filter";

#[derive(Copy, Clone)]
pub struct MultiSelectOptions<'a> {
    message: &'a str,
    options: &'a [&'a str],
    default: Option<&'a [usize]>,
    help_message: &'a str,
    page_size: usize,
    vim_mode: bool,
    starting_selection: usize,
    filter: &'a Filter,
    keep_filter: bool,
    transformer: &'a Transformer,
}

impl<'a> MultiSelectOptions<'a> {
    pub fn new(message: &'a str, options: &'a [&str]) -> Result<Self, Box<dyn Error>> {
        if options.is_empty() {
            bail!("Please provide options to select from");
        }

        Ok(Self {
            message,
            options,
            default: None,
            help_message: DEFAULT_HELP_MESSAGE,
            page_size: DEFAULT_PAGE_SIZE,
            vim_mode: DEFAULT_VIM_MODE,
            starting_selection: DEFAULT_STARTING_SELECTION,
            keep_filter: DEFAULT_KEEP_FILTER,
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

    pub fn with_keep_filter(mut self, keep_filter: bool) -> Self {
        self.keep_filter = keep_filter;
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

    pub fn with_default(mut self, default: &'a [usize]) -> Result<Self, Box<dyn Error>> {
        for i in default {
            if i >= &self.options.len() {
                bail!("Invalid index, larger than options available");
            }
        }

        self.default = Some(default);
        Ok(self)
    }

    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Result<Self, Box<dyn Error>> {
        if starting_cursor >= self.options.len() {
            bail!("Starting selection should not be larger than length of options");
        }

        self.starting_selection = starting_cursor;
        Ok(self)
    }
}

impl<'a> QuestionOptions<'a> for MultiSelectOptions<'a> {
    fn with_config(mut self, global_config: &'a PromptConfig) -> Self {
        if let Some(page_size) = global_config.page_size {
            self.page_size = page_size;
        }
        if let Some(vim_mode) = global_config.vim_mode {
            self.vim_mode = vim_mode;
        }
        if let Some(keep_filter) = global_config.keep_filter {
            self.keep_filter = keep_filter;
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
}

pub(in crate) struct MultiSelect<'a> {
    message: &'a str,
    options: &'a [&'a str],
    help_message: &'a str,
    vim_mode: bool,
    cursor_index: usize,
    checked: HashSet<usize>,
    page_size: usize,
    renderer: Renderer,
    keep_filter: bool,
    filter_value: Option<String>,
    filtered_options: Vec<usize>,
    filter: &'a Filter,
    transformer: &'a Transformer,
}

impl<'a> From<MultiSelectOptions<'a>> for MultiSelect<'a> {
    fn from(mso: MultiSelectOptions<'a>) -> Self {
        Self {
            message: mso.message,
            options: mso.options,
            help_message: mso.help_message,
            vim_mode: mso.vim_mode,
            cursor_index: mso.starting_selection,
            renderer: Renderer::default(),
            page_size: mso.page_size,
            keep_filter: mso.keep_filter,
            filter_value: None,
            filtered_options: Vec::from_iter(0..mso.options.len()),
            filter: mso.filter,
            transformer: mso.transformer,
            checked: mso
                .default
                .map_or_else(|| HashSet::new(), |d| d.iter().cloned().collect()),
        }
    }
}

impl<'a> MultiSelect<'a> {
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

    fn get_final_answer(&self) -> Result<Answer, Box<dyn Error>> {
        Ok(Answer::MultipleOptions(
            self.options
                .iter()
                .enumerate()
                .filter_map(|(idx, opt)| match &self.checked.contains(&idx) {
                    true => Some(OptionAnswer::new(idx, opt)),
                    false => None,
                })
                .collect::<Vec<OptionAnswer>>(),
        ))
    }

    fn cleanup(&mut self, terminal: &mut Terminal, answer: &str) -> Result<(), Box<dyn Error>> {
        self.renderer.reset_prompt(terminal)?;
        self.renderer
            .print_prompt_answer(terminal, &self.message, answer)?;

        Ok(())
    }
}

impl<'a> Prompt for MultiSelect<'a> {
    fn render(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        self.renderer.reset_prompt(terminal)?;

        if let Some(filter) = &self.filter_value {
            self.renderer
                .print_prompt_with_content(terminal, &prompt, filter)?;
        } else {
            self.renderer.print_prompt(terminal, &prompt)?;
        }

        let choices = self
            .filtered_options
            .iter()
            .cloned()
            .map(|i| OptionAnswer::new(i, self.options.get(i).unwrap()))
            .collect::<Vec<OptionAnswer>>();

        let (paginated_opts, rel_sel) = paginate(self.page_size, &choices, self.cursor_index);

        for (idx, opt) in paginated_opts.iter().enumerate() {
            self.renderer.print_multi_option(
                terminal,
                rel_sel == idx,
                self.checked.contains(&opt.index),
                &opt.value,
            )?;
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
