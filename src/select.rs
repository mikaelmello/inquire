use simple_error::SimpleError;
use std::{error::Error, iter::FromIterator};
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    config::PromptConfig,
    question::{Answer, Question},
    renderer::Renderer,
    survey::{paginate, OptionAnswer},
    terminal::Terminal,
};

pub struct Select<'a> {
    message: String,
    options: Vec<&'a str>,
    help: Option<&'a str>,
    config: Box<PromptConfig>,
    vim_mode: bool,
    filter_value: Option<String>,
    filtered_options: Vec<usize>,
    selected_index: usize,
    initialized: bool,
    final_answer: Option<String>,
    renderer: Renderer,
}

impl<'a> Select<'a> {
    pub fn new(message: &str, options: &'a [&str]) -> Result<Self, Box<dyn Error>> {
        if options.is_empty() {
            bail!("Please provide options to select from");
        }

        Ok(Self {
            message: message.to_string(),
            options: Vec::from(options),
            help: None,
            config: Box::new(PromptConfig::default()),
            vim_mode: false,
            filter_value: None,
            filtered_options: Vec::from_iter(0..options.len()),
            selected_index: 0,
            initialized: false,
            final_answer: None,
            renderer: Renderer::default(),
        })
    }

    pub fn with_default(mut self, index: usize) -> Result<Self, Box<dyn Error>> {
        if index >= self.options.len() {
            bail!("Invalid index, larger than options available");
        }

        self.selected_index = index;

        Ok(self)
    }

    pub fn with_help(mut self, help: &'a str) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    pub fn with_config(mut self, c: Box<PromptConfig>) -> Self {
        self.config = c;
        self
    }

    fn filter_options(&self) -> Vec<usize> {
        let filter = &self.config.filter;

        self.options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| match &self.filter_value {
                Some(val) if filter(&val, opt, i) => Some(i),
                Some(_) => None,
                None => Some(i),
            })
            .collect()
    }

    fn move_cursor_up(&mut self) {
        self.selected_index = self
            .selected_index
            .checked_sub(1)
            .unwrap_or_else(|| self.filtered_options.len() - 1);
    }

    fn move_cursor_down(&mut self) {
        self.selected_index = self.selected_index.saturating_add(1);
        if self.selected_index == self.filtered_options.len() {
            self.selected_index = 0;
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
            if options.len() > 0 && options.len() <= self.selected_index {
                self.selected_index = options.len().saturating_sub(1);
            }
            self.filtered_options = options;
        }
    }

    fn get_final_answer(&self) -> Result<Answer, Box<dyn Error>> {
        self.filtered_options
            .get(self.selected_index)
            .and_then(|i| self.options.get(*i).map(|opt| OptionAnswer::new(*i, opt)))
            .map(|o| Answer::Option(o))
            .ok_or(Box::new(SimpleError::new("Invalid selected index")))
    }
}

impl<'a> Question for Select<'a> {
    fn render(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        self.renderer.reset_prompt(terminal)?;

        if let Some(final_answer) = &self.final_answer {
            self.renderer
                .print_prompt_answer(terminal, &prompt, &final_answer)?;
            return Ok(());
        }

        if let Some(filter) = &self.filter_value {
            self.renderer
                .print_prompt_filter(terminal, &prompt, filter)?;
        } else {
            self.renderer.print_prompt(terminal, &prompt)?;
        }

        let choices = self
            .filtered_options
            .iter()
            .cloned()
            .map(|i| OptionAnswer::new(i, self.options.get(i).unwrap()))
            .collect::<Vec<OptionAnswer>>();

        let (paginated_opts, rel_sel) =
            paginate(self.config.page_size, &choices, self.selected_index);

        for (idx, opt) in paginated_opts.iter().enumerate() {
            self.renderer
                .print_option(terminal, rel_sel == idx, &opt.value)?;
        }

        self.renderer.print_help(
            terminal,
            "↑↓ to move, space or enter to select, type to filter",
        )?;

        Ok(())
    }

    fn cleanup(&mut self, answer: &Answer) -> Result<(), Box<dyn Error>> {
        self.final_answer = Some(answer.to_string());

        let mut terminal = Terminal::new()?;
        terminal.cursor_hide()?;

        self.render(&mut terminal)?;

        terminal.cursor_show()?;
        Ok(())
    }

    fn prompt(&mut self) -> Result<Answer, Box<dyn Error>> {
        // TODO: improve state machine
        if self.initialized {
            bail!("Question was already prompted");
        }
        self.initialized = true;

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

        terminal.cursor_show()?;

        self.get_final_answer()
    }
}
