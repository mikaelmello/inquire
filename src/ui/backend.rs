use std::{fmt::Display, io::Result};

use super::{key::Key, RenderConfig, Terminal};
use crate::{
    input::Input,
    ui::{Color, Styled},
};

pub trait CommonBackend {
    fn read_key(&mut self) -> Result<Key>;

    fn frame_setup(&mut self) -> Result<()>;
    fn frame_finish(&mut self) -> Result<()>;

    fn finish_prompt(&mut self, prompt: &str, answer: &str) -> Result<()>;

    fn render_error_message(&mut self, error: &str) -> Result<()>;
    fn render_help_message(&mut self, help: &str) -> Result<()>;
}

pub trait TextBackend: CommonBackend {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> Result<()>;
    fn render_suggestion<T: Display>(&mut self, content: T, focused: bool) -> Result<()>;
}

pub trait SelectBackend: CommonBackend {
    fn render_select_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
    fn render_option<T: Display>(&mut self, content: T, focused: bool) -> Result<()>;
}

pub trait MultiSelectBackend: CommonBackend {
    fn render_multiselect_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
    fn render_option<T: Display>(&mut self, content: T, focused: bool, checked: bool)
        -> Result<()>;
}

#[cfg(feature = "date")]
pub trait DateSelectBackend: CommonBackend {
    fn render_calendar_prompt(&mut self, prompt: &str) -> Result<()>;
    fn render_calendar(
        &mut self,
        month: chrono::Month,
        year: i32,
        week_start: chrono::Weekday,
        today: chrono::NaiveDate,
        selected_date: chrono::NaiveDate,
        min_date: Option<chrono::NaiveDate>,
        max_date: Option<chrono::NaiveDate>,
    ) -> Result<()>;
}

pub trait CustomTypeBackend: CommonBackend {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> Result<()>;
}

pub trait PasswordBackend: CommonBackend {
    fn render_password_prompt(&mut self, prompt: &str) -> Result<()>;
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
    pub fn new(terminal: T, render_config: &'a RenderConfig) -> Result<Self> {
        let mut backend = Self {
            cur_line: 0,
            terminal,
            render_config,
        };

        backend.terminal.cursor_hide()?;

        Ok(backend)
    }

    fn reset_prompt(&mut self) -> Result<()> {
        for _ in 0..self.cur_line {
            self.terminal.cursor_up()?;
            self.terminal.cursor_move_to_column(0)?;
            self.terminal.clear_current_line()?;
        }

        self.cur_line = 0;
        Ok(())
    }

    fn print_prompt_prefix(&mut self) -> Result<()> {
        self.terminal
            .write_styled(&self.render_config.prompt_prefix)
    }

    fn print_prompt_token(&mut self, prompt: &str) -> Result<()> {
        let token = Styled::new(prompt).with_style_sheet(self.render_config.prompt);

        self.terminal.write_styled(&token)
    }

    fn print_default_value(&mut self, value: &str) -> Result<()> {
        let content = format!("({})", value);
        let token = Styled::new(content).with_style_sheet(self.render_config.default_value);

        self.terminal.write_styled(&token)
    }

    fn print_prompt(&mut self, prompt: &str) -> Result<()> {
        self.print_prompt_prefix()?;

        self.terminal.write(' ')?;

        self.print_prompt_token(prompt)?;

        Ok(())
    }

    fn print_prompt_with_answer(&mut self, prompt: &str, answer: &str) -> Result<()> {
        self.print_prompt(prompt)?;

        self.terminal.write(' ')?;

        let token = Styled::new(answer).with_style_sheet(self.render_config.answer);
        self.terminal.write_styled(&token)?;

        self.new_line()?;

        Ok(())
    }

    fn print_input(&mut self, input: &Input) -> Result<()> {
        let (before, mut at, after) = input.split();

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

        Ok(())
    }

    fn print_prompt_with_input(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        input: &Input,
    ) -> Result<()> {
        self.print_prompt(prompt)?;

        if let Some(default) = default {
            self.terminal.write(' ')?;
            self.print_default_value(default)?;
        }

        self.print_input(input)?;

        self.new_line()?;

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.terminal.flush()?;

        Ok(())
    }

    fn new_line(&mut self) -> Result<()> {
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
    fn frame_setup(&mut self) -> Result<()> {
        self.reset_prompt()
    }

    fn frame_finish(&mut self) -> Result<()> {
        self.flush()
    }

    fn finish_prompt(&mut self, prompt: &str, answer: &str) -> Result<()> {
        self.reset_prompt()?;
        self.print_prompt_with_answer(prompt, answer)?;

        Ok(())
    }

    fn read_key(&mut self) -> Result<Key> {
        self.terminal.read_key().map(Key::from)
    }

    fn render_error_message(&mut self, error: &str) -> Result<()> {
        self.terminal
            .write_styled(&self.render_config.error_message.prefix)?;

        self.terminal.write_styled(
            &Styled::new(' ').with_style_sheet(self.render_config.error_message.separator),
        )?;

        self.terminal.write_styled(
            &Styled::new(error).with_style_sheet(self.render_config.error_message.message),
        )?;

        self.new_line()?;

        Ok(())
    }

    fn render_help_message(&mut self, help: &str) -> Result<()> {
        self.terminal
            .write_styled(&Styled::new('[').with_style_sheet(self.render_config.help_message))?;

        self.terminal
            .write_styled(&Styled::new(help).with_style_sheet(self.render_config.help_message))?;

        self.terminal
            .write_styled(&Styled::new(']').with_style_sheet(self.render_config.help_message))?;

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
    ) -> Result<()> {
        self.print_prompt_with_input(prompt, default, cur_input)
    }

    fn render_suggestion<D: Display>(&mut self, content: D, focused: bool) -> Result<()> {
        let token = match focused {
            true => Styled::new(format!("> {}", content)).with_fg(Color::Cyan),
            false => Styled::new(format!("  {}", content)),
        };

        self.terminal.write_styled(&token)?;

        self.new_line()?;

        Ok(())
    }
}

impl<'a, T> SelectBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_select_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()> {
        self.print_prompt_with_input(prompt, None, cur_input)
    }

    fn render_option<D: Display>(&mut self, content: D, focused: bool) -> Result<()> {
        let token = match focused {
            true => Styled::new(format!("> {}", content)).with_fg(Color::Cyan),
            false => Styled::new(format!("  {}", content)),
        };

        self.terminal.write_styled(&token)?;

        self.new_line()?;

        Ok(())
    }
}

impl<'a, T> MultiSelectBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_multiselect_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()> {
        self.print_prompt_with_input(prompt, None, cur_input)
    }

    fn render_option<D: Display>(
        &mut self,
        content: D,
        focused: bool,
        checked: bool,
    ) -> Result<()> {
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
    fn render_calendar_prompt(&mut self, prompt: &str) -> Result<()> {
        self.print_prompt(prompt)?;
        self.new_line()?;
        Ok(())
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
    ) -> Result<()> {
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
    ) -> Result<()> {
        self.print_prompt_with_input(prompt, default, cur_input)
    }
}

impl<'a, T> PasswordBackend for Backend<'a, T>
where
    T: Terminal,
{
    fn render_password_prompt(&mut self, prompt: &str) -> Result<()> {
        self.print_prompt(prompt)?;
        self.new_line()?;
        Ok(())
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
