use std::{collections::HashSet, error::Error};
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    config::PromptConfig,
    question::Question,
    Answer::{self},
};

struct MultiSelect<'a> {
    message: String,
    options: Vec<&'a str>,
    default: Option<Vec<usize>>,
    help: Option<&'a str>,
    page_size: usize,
    vim_mode: bool,
    filter: fn(filter: &str, value: &str, index: usize) -> bool,
    filter_value: Option<String>,
    filtered_options: Vec<(usize, &'a str)>,
    selected_index: usize,
    checked: HashSet<usize>,
    showing_help: bool,
    initialized: bool,
}

impl<'a> MultiSelect<'a> {
    pub fn new(message: &str, options: &'a [&str]) -> Result<Self, Box<dyn Error>> {
        let default_config = PromptConfig::default();

        if options.is_empty() {
            bail!("Please provide options to select from");
        }

        Ok(Self {
            message: message.to_string(),
            options: Vec::from(options),
            default: None,
            help: None,
            page_size: default_config.page_size,
            vim_mode: false,
            filter: default_config.filter,
            filter_value: None,
            filtered_options: vec![],
            selected_index: 0,
            checked: HashSet::new(),
            showing_help: false,
            initialized: false,
        })
    }

    pub fn with_default(mut self, indexes: &[usize]) -> Result<Self, Box<dyn Error>> {
        for i in indexes {
            if i >= &self.options.len() {
                bail!("Invalid index, larger than options available");
            }
            self.checked.insert(*i);
        }

        self.default = Some(indexes.iter().cloned().collect());

        Ok(self)
    }

    pub fn with_help(mut self, help: &'a str) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Result<Self, Box<dyn Error>> {
        if page_size == 0 {
            bail!("Page size must be larger than 0");
        }
        self.page_size = page_size;

        Ok(self)
    }

    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    pub fn with_filter(mut self, f: fn(filter: &str, value: &str, index: usize) -> bool) -> Self {
        self.filter = f;
        self
    }

    fn filter_options(&self, config: &PromptConfig) -> Vec<(usize, &'a str)> {
        let mut answers = Vec::new();

        let filter_value = match &self.filter_value {
            Some(val) => val,
            None => return self.options.iter().cloned().enumerate().collect(),
        };

        let filter = self.filter;

        for (i, opt) in self.options.iter().enumerate() {
            if filter(&filter_value, opt, i) {
                answers.push((i, *opt));
            }
        }

        answers
    }

    fn on_change(&mut self, key: Key, config: &PromptConfig) {
        let mut options = self.filter_options(config);
        let old_filter = self.filter_value.clone();

        match key {
            Key::Up | Key::Char('k') if self.vim_mode => {
                self.selected_index = self
                    .selected_index
                    .checked_sub(1)
                    .unwrap_or_else(|| options.len());
            }
            Key::Char('\t') | Key::Down | Key::Char('j') if self.vim_mode => {
                self.selected_index = self.selected_index.saturating_add(1);
                if self.selected_index == options.len() {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
            }
            Key::Char(' ') => {
                let (idx, _) = match options.get(self.selected_index) {
                    Some(val) => val,
                    None => return,
                };

                if self.checked.contains(idx) {
                    self.checked.remove(idx);
                } else {
                    self.checked.insert(*idx);
                }

                if !config.keep_filter {
                    self.filter_value = None;
                }
            }
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
                for (i, _) in &options {
                    self.checked.insert(*i);
                }

                if !config.keep_filter {
                    self.filter_value = None;
                }
            }
            Key::Left => {
                self.checked.clear();

                if !config.keep_filter {
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
            options = self.filter_options(config);
            if options.len() > 0 && options.len() <= self.selected_index {
                self.selected_index = options.len().saturating_sub(1);
            }
        }

        self.filtered_options = options;
        self.render();
    }
}

impl<'a> Question for MultiSelect<'a> {
    fn render(&self) {}

    fn cleanup(&mut self) {
        todo!()
    }

    fn prompt(&mut self, config: &PromptConfig) -> Result<Answer, Box<dyn Error>> {
        // TODO: improve state machine
        if self.initialized {
            bail!("Question was already prompted");
        }
        self.initialized = true;

        return Ok(Answer::Simple(String::from("place_holder")));
    }
}
