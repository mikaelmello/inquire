use chrono::{Datelike, NaiveDate};
use std::{
    error::Error,
    ops::{Add, Sub},
};
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    config::{self, Filter},
    date_utils::get_month,
    formatter::{self, DateFormatter},
    renderer::Renderer,
    terminal::Terminal,
};

/// Presents a message to the user and a date picker for the user to choose from.
#[derive(Copy, Clone)]
pub struct DateSelect<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Default date to be selected.
    pub default: NaiveDate,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Whether vim mode is enabled. When enabled, the user can
    /// navigate through the options using hjkl.
    pub vim_mode: bool,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: DateFormatter,
}

impl<'a> DateSelect<'a> {
    /// Default formatter.
    pub const DEFAULT_FORMATTER: DateFormatter = formatter::DEFAULT_DATE_FORMATTER;
    /// Default filter.
    pub const DEFAULT_FILTER: Filter = config::DEFAULT_FILTER;
    /// Default value of vim mode.
    pub const DEFAULT_VIM_MODE: bool = config::DEFAULT_VIM_MODE;
    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("↑↓ to move, space or enter to select, type to filter");

    /// Creates a [Date] with the provided message and options, along with default configuration values.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            default: chrono::Local::now().date().naive_local(),
            help_message: Self::DEFAULT_HELP_MESSAGE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            formatter: Self::DEFAULT_FORMATTER,
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Removes the set help message.
    pub fn without_help_message(mut self) -> Self {
        self.help_message = None;
        self
    }

    /// Sets the default date.
    pub fn with_default(mut self, default: NaiveDate) -> Self {
        self.default = default;
        self
    }

    /// Enables or disabled vim_mode.
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: DateFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to them.
    pub fn prompt(self) -> Result<NaiveDate, Box<dyn Error>> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(
        self,
        renderer: &mut Renderer,
    ) -> Result<NaiveDate, Box<dyn Error>> {
        DateSelectPrompt::new(self).prompt(renderer)
    }
}

struct DateSelectPrompt<'a> {
    message: &'a str,
    current_date: NaiveDate,
    help_message: Option<&'a str>,
    vim_mode: bool,
    filter_value: Option<String>,
    formatter: DateFormatter,
    error: Option<String>,
}

impl<'a> DateSelectPrompt<'a> {
    fn new(so: DateSelect<'a>) -> Self {
        Self {
            message: so.message,
            current_date: so.default,
            help_message: so.help_message,
            vim_mode: so.vim_mode,
            filter_value: None,
            formatter: so.formatter,
            error: None,
        }
    }

    fn move_cursor_up(&mut self) {
        self.current_date = self.current_date.sub(chrono::Duration::weeks(1));
    }

    fn move_cursor_down(&mut self) {
        self.current_date = self.current_date.add(chrono::Duration::weeks(1));
    }

    fn move_cursor_left(&mut self) {
        self.current_date = self.current_date.sub(chrono::Duration::days(1));
    }

    fn move_cursor_right(&mut self) {
        self.current_date = self.current_date.add(chrono::Duration::days(1));
    }

    fn on_change(&mut self, key: Key) {
        match key {
            Key::Up => self.move_cursor_up(),
            Key::Char('k') if self.vim_mode => self.move_cursor_up(),
            Key::Char('\t') | Key::Down => self.move_cursor_down(),
            Key::Char('j') if self.vim_mode => self.move_cursor_down(),
            Key::Left => self.move_cursor_left(),
            Key::Char('h') if self.vim_mode => self.move_cursor_left(),
            Key::Right => self.move_cursor_right(),
            Key::Char('l') if self.vim_mode => self.move_cursor_right(),
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
    }

    fn get_final_answer(&self) -> Result<NaiveDate, String> {
        Ok(self.current_date)
    }

    fn render(&mut self, renderer: &mut Renderer) -> Result<(), std::io::Error> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(err) = &self.error {
            renderer.print_error_message(err)?;
        }

        renderer.print_prompt(&prompt, None, self.filter_value.as_deref())?;

        renderer.print_calendar_month(
            get_month(self.current_date.month()),
            self.current_date.year(),
            chrono::Weekday::Sun,
            chrono::Local::now().date().naive_local(),
            self.current_date,
        )?;

        if let Some(help_message) = self.help_message {
            renderer.print_help(help_message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> Result<NaiveDate, Box<dyn Error>> {
        let final_answer: NaiveDate;

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

        let formatted = (self.formatter)(&final_answer);

        renderer.cleanup(&self.message, &formatted)?;

        Ok(final_answer)
    }
}
