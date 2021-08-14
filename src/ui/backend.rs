use std::fmt::Display;

use super::{key::Key, RenderConfig, Terminal};
use crate::{
    error::{InquireError, InquireResult},
    input::Input,
    ui::{Attributes, Color, Styled},
};

pub trait CommonBackend {
    fn read_key(&mut self) -> InquireResult<Key>;

    fn frame_setup(&mut self) -> InquireResult<()>;
    fn frame_finish(&mut self) -> InquireResult<()>;

    fn finish_prompt(&mut self, prompt: &str, answer: &str) -> InquireResult<()>;

    fn render_error_message(&mut self, error: &str) -> InquireResult<()>;
    fn render_help_message(&mut self, help: &str) -> InquireResult<()>;
}

pub trait TextBackend: CommonBackend {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> InquireResult<()>;
    fn render_suggestion<T: Display>(&mut self, content: T, focused: bool) -> InquireResult<()>;
}

pub trait SelectBackend: CommonBackend {
    fn render_select_prompt(&mut self, prompt: &str, cur_input: &Input) -> InquireResult<()>;
    fn render_option<T: Display>(&mut self, content: T, focused: bool) -> InquireResult<()>;
}

pub trait MultiSelectBackend: CommonBackend {
    fn render_multiselect_prompt(&mut self, prompt: &str, cur_input: &Input) -> InquireResult<()>;
    fn render_option<T: Display>(
        &mut self,
        content: T,
        focused: bool,
        checked: bool,
    ) -> InquireResult<()>;
}

#[cfg(feature = "date")]
pub trait DateSelectBackend: CommonBackend {
    fn render_calendar_prompt(&mut self, prompt: &str) -> InquireResult<()>;
    fn render_calendar(
        &mut self,
        month: chrono::Month,
        year: i32,
        week_start: chrono::Weekday,
        today: chrono::NaiveDate,
        selected_date: chrono::NaiveDate,
        min_date: Option<chrono::NaiveDate>,
        max_date: Option<chrono::NaiveDate>,
    ) -> InquireResult<()>;
}

pub trait CustomTypeBackend: CommonBackend {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> InquireResult<()>;
}

pub trait PasswordBackend: CommonBackend {
    fn render_password_prompt(&mut self, prompt: &str) -> InquireResult<()>;
}

pub struct Backend<'a, T>
where
    T: Terminal,
{
    cur_line: usize,
    terminal: T,
    render_config: &'a RenderConfig,
}

impl<'a, T> Backend<'a, T>
where
    T: Terminal,
{
    pub fn new(terminal: T, render_config: &'a RenderConfig) -> InquireResult<Self> {
        let mut backend = Self {
            cur_line: 0,
            terminal,
            render_config,
        };

        backend.terminal.cursor_hide()?;

        Ok(backend)
    }

    pub fn reset_prompt(&mut self) -> InquireResult<()> {
        for _ in 0..self.cur_line {
            self.terminal.cursor_up()?;
            self.terminal.cursor_move_to_column(0)?;
            self.terminal.clear_current_line()?;
        }

        self.cur_line = 0;
        Ok(())
    }

    fn print_prompt_prefix(&mut self) -> InquireResult<()> {
        self.terminal
            .write_styled(&self.render_config.prompt_prefix)?;
        self.terminal.write(' ')?;
        Ok(())
    }

    fn print_prompt_token(&mut self, prompt: &str) -> InquireResult<()> {
        let token = Styled::new(prompt).with_style_sheet(self.render_config.prompt);
        self.terminal.write_styled(&token)?;
        Ok(())
    }

    fn print_prompt_answer(&mut self, prompt: &str, answer: &str) -> InquireResult<()> {
        self.print_prompt_prefix()?;

        self.print_prompt_token(prompt)?;

        self.terminal
            .write_styled(&Styled::new(format!(" {}", answer)).with_fg(Color::Cyan))?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        content: Option<&str>,
    ) -> InquireResult<()> {
        self.print_prompt_prefix()?;

        self.print_prompt_token(prompt)?;

        if let Some(default) = default {
            self.terminal.write(format!(" ({})", default))?;
        }

        match content {
            Some(content) if !content.is_empty() => self
                .terminal
                .write_styled(&Styled::new(format!(" {}", content)).with_attr(Attributes::BOLD))?,
            _ => {}
        }

        self.new_line()?;

        Ok(())
    }

    pub fn print_prompt_input(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        content: &Input,
    ) -> InquireResult<()> {
        self.print_prompt_prefix()?;

        self.print_prompt_token(prompt)?;

        if let Some(default) = default {
            self.terminal.write(format!(" ({})", default))?;
        }

        let (before, mut at, after) = content.split();

        if at.is_empty() {
            at.push(' ');
        }

        self.terminal.write(' ')?;

        self.terminal.write_styled(
            &Styled::new(before).with_style_sheet(self.render_config.text_input.text),
        )?;
        self.terminal.write_styled(
            &Styled::new(at).with_style_sheet(self.render_config.text_input.cursor),
        )?;
        self.terminal.write_styled(
            &Styled::new(after).with_style_sheet(self.render_config.text_input.text),
        )?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_option<D: Display>(&mut self, content: D, focused: bool) -> InquireResult<()> {
        let token = match focused {
            true => Styled::new(format!("> {}", content)).with_fg(Color::Cyan),
            false => Styled::new(format!("  {}", content)),
        };

        self.terminal.write_styled(&token)?;

        self.new_line()?;

        Ok(())
    }

    pub fn flush(&mut self) -> InquireResult<()> {
        self.terminal.flush()?;

        Ok(())
    }

    fn new_line(&mut self) -> InquireResult<()> {
        self.terminal.cursor_move_to_column(0)?;
        self.terminal.write("\n")?;
        self.cur_line = self.cur_line.saturating_add(1);

        Ok(())
    }
}

impl<'a, T> CommonBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn frame_setup(&mut self) -> InquireResult<()> {
        self.reset_prompt()
    }

    fn frame_finish(&mut self) -> InquireResult<()> {
        self.flush()
    }

    fn finish_prompt(&mut self, prompt: &str, answer: &str) -> InquireResult<()> {
        self.reset_prompt()?;
        self.print_prompt_answer(prompt, answer)?;

        Ok(())
    }

    fn read_key(&mut self) -> InquireResult<Key> {
        self.terminal
            .read_key()
            .map(Key::from)
            .map_err(InquireError::from)
    }

    fn render_error_message(&mut self, error: &str) -> InquireResult<()> {
        self.terminal
            .write_styled(&Styled::new(format!("# {}", error)).with_fg(Color::Red))?;

        self.new_line()?;

        Ok(())
    }

    fn render_help_message(&mut self, help: &str) -> InquireResult<()> {
        self.terminal
            .write_styled(&Styled::new(format!("[{}]", help)).with_fg(Color::Cyan))?;

        self.new_line()?;

        Ok(())
    }
}

impl<'a, T> TextBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> InquireResult<()> {
        self.print_prompt_input(prompt, default, cur_input)
    }

    fn render_suggestion<D: Display>(&mut self, content: D, focused: bool) -> InquireResult<()> {
        self.print_option(content, focused)
    }
}

impl<'a, T> SelectBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_select_prompt(&mut self, prompt: &str, cur_input: &Input) -> InquireResult<()> {
        self.print_prompt_input(prompt, None, cur_input)
    }

    fn render_option<D: Display>(&mut self, content: D, focused: bool) -> InquireResult<()> {
        self.print_option(content, focused)
    }
}

impl<'a, T> MultiSelectBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_multiselect_prompt(&mut self, prompt: &str, cur_input: &Input) -> InquireResult<()> {
        self.print_prompt_input(prompt, None, cur_input)
    }

    fn render_option<D: Display>(
        &mut self,
        content: D,
        focused: bool,
        checked: bool,
    ) -> InquireResult<()> {
        let cursor = match focused {
            true => Styled::new("> ").with_fg(Color::Cyan),
            false => Styled::new("  "),
        };

        let checkbox = match checked {
            true => Styled::new("[x] ").with_fg(Color::Green),
            false => Styled::new("[ ] "),
        };

        self.terminal.write_styled(&cursor)?;
        self.terminal.write_styled(&checkbox)?;
        self.terminal.write(content)?;

        self.new_line()?;

        Ok(())
    }
}

#[cfg(feature = "date")]
impl<'a, T> DateSelectBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_calendar_prompt(&mut self, prompt: &str) -> InquireResult<()> {
        self.print_prompt(prompt, None, None)
    }

    fn render_calendar(
        &mut self,
        month: chrono::Month,
        year: i32,
        week_start: chrono::Weekday,
        today: chrono::NaiveDate,
        selected_date: chrono::NaiveDate,
        min_date: Option<chrono::NaiveDate>,
        max_date: Option<chrono::NaiveDate>,
    ) -> InquireResult<()> {
        use crate::date_utils::get_start_date;
        use chrono::{Datelike, Duration};
        use std::ops::Sub;

        // print header (month year)
        let header = format!("{} {}", month.name().to_lowercase(), year);

        self.terminal
            .write_styled(&Styled::new("> ").with_fg(Color::Green))?;

        self.terminal.write(format!("{:^20}", header))?;

        self.new_line()?;

        // print week header
        let mut current_weekday = week_start;
        let mut week_days: Vec<String> = vec![];
        for _ in 0..7 {
            let mut formatted = format!("{}", current_weekday);
            formatted.make_ascii_lowercase();
            formatted.pop();
            week_days.push(formatted);

            current_weekday = current_weekday.succ();
        }
        let week_days = week_days.join(" ");

        self.terminal
            .write_styled(&Styled::new("> ").with_fg(Color::Green))?;

        self.terminal.write(week_days)?;
        self.new_line()?;

        // print dates
        let mut date_it = get_start_date(month, year);
        // first date of week-line is possibly in the previous month
        if date_it.weekday() == week_start {
            date_it = date_it.sub(Duration::weeks(1));
        } else {
            while date_it.weekday() != week_start {
                date_it = date_it.pred();
            }
        }

        for _ in 0..6 {
            self.terminal
                .write_styled(&Styled::new("> ").with_fg(Color::Green))?;

            for i in 0..7 {
                if i > 0 {
                    self.terminal.write(" ")?;
                }

                let date = format!("{:2}", date_it.day());

                let mut token = Styled::new(date);

                if date_it == selected_date {
                    token = token.with_bg(Color::Grey).with_fg(Color::Black);
                } else if date_it == today {
                    token = token.with_fg(Color::Green);
                } else if date_it.month() != month.number_from_month() {
                    token = token.with_fg(Color::DarkGrey);
                }

                if let Some(min_date) = min_date {
                    if date_it < min_date {
                        token = token.with_fg(Color::DarkGrey);
                    }
                }

                if let Some(max_date) = max_date {
                    if date_it > max_date {
                        token = token.with_fg(Color::DarkGrey);
                    }
                }

                self.terminal.write_styled(&token)?;

                date_it = date_it.succ();
            }

            self.new_line()?;
        }

        Ok(())
    }
}

impl<'a, T> CustomTypeBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> InquireResult<()> {
        self.print_prompt_input(prompt, default, cur_input)
    }
}

impl<'a, T> PasswordBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_password_prompt(&mut self, prompt: &str) -> InquireResult<()> {
        self.print_prompt(prompt, None, None)
    }
}

impl<'a, T> Drop for Backend<'a, T>
where
    T: Terminal,
{
    fn drop(&mut self) {
        let _ = self.terminal.cursor_show();
    }
}
