use super::{key::Key, terminal::Terminal};
use crate::{
    error::{InquireError, InquireResult},
    input::Input,
    ui::{
        color::Color,
        style::{Attributes, Styled},
    },
};

pub struct Renderer<'a> {
    cur_line: usize,
    terminal: Terminal<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(terminal: Terminal<'a>) -> InquireResult<Self> {
        let mut renderer = Self {
            cur_line: 0,
            terminal,
        };

        renderer.terminal.cursor_hide()?;

        Ok(renderer)
    }

    pub fn reset_prompt(&mut self) -> InquireResult<()> {
        for _ in 0..self.cur_line {
            self.terminal.cursor_up()?;
            self.terminal.cursor_horizontal_reset()?;
            self.terminal.clear_current_line()?;
        }

        self.cur_line = 0;
        Ok(())
    }

    pub fn print_error_message(&mut self, message: &str) -> InquireResult<()> {
        self.terminal
            .write_styled(Styled::new(format!("# {}", message)).with_fg(Color::Red))?;

        self.new_line()?;

        Ok(())
    }

    fn print_prompt_answer(&mut self, prompt: &str, answer: &str) -> InquireResult<()> {
        self.terminal
            .write_styled(Styled::new("? ").with_fg(Color::Green))?;

        self.terminal.write(prompt)?;

        self.terminal
            .write_styled(Styled::new(format!(" {}", answer)).with_fg(Color::Cyan))?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        content: Option<&str>,
    ) -> InquireResult<()> {
        self.terminal
            .write_styled(Styled::new("? ").with_fg(Color::Green))?;

        self.terminal.write(prompt)?;

        if let Some(default) = default {
            self.terminal.write(format!(" ({})", default))?;
        }

        match content {
            Some(content) if !content.is_empty() => self
                .terminal
                .write_styled(Styled::new(format!(" {}", content)).with_attr(Attributes::BOLD))?,
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
        self.terminal
            .write_styled(Styled::new("? ").with_fg(Color::Green))?;

        self.terminal.write(prompt)?;

        if let Some(default) = default {
            self.terminal.write(format!(" ({})", default))?;
        }

        let (before, mut at, after) = content.split();

        if at.is_empty() {
            at.push(' ');
        }

        self.terminal.write(" ")?;
        self.terminal.write(before)?;
        self.terminal
            .write_styled(Styled::new(at).with_bg(Color::Grey).with_fg(Color::Black))?;
        self.terminal.write(after)?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_help(&mut self, message: &str) -> InquireResult<()> {
        self.terminal
            .write_styled(Styled::new(format!("[{}]", message)).with_fg(Color::Cyan))?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_option(&mut self, cursor: bool, content: &str) -> InquireResult<()> {
        let token = match cursor {
            true => Styled::new(format!("> {}", content)).with_fg(Color::Cyan),
            false => Styled::new(format!("  {}", content)),
        };

        self.terminal.write_styled(token)?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_multi_option(
        &mut self,
        cursor: bool,
        checked: bool,
        content: &str,
    ) -> InquireResult<()> {
        let cursor = match cursor {
            true => Styled::new("> ").with_fg(Color::Cyan),
            false => Styled::new("  "),
        };

        let checked = match checked {
            true => Styled::new("[x] ").with_fg(Color::Green),
            false => Styled::new("[ ] "),
        };

        self.terminal.write_styled(cursor)?;
        self.terminal.write_styled(checked)?;
        self.terminal.write(content)?;

        self.new_line()?;

        Ok(())
    }

    #[cfg(feature = "date")]
    pub fn print_calendar_month(
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
            .write_styled(Styled::new("> ").with_fg(Color::Green))?;

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
            .write_styled(Styled::new("> ").with_fg(Color::Green))?;

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
                .write_styled(Styled::new("> ").with_fg(Color::Green))?;

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

                self.terminal.write_styled(token)?;

                date_it = date_it.succ();
            }

            self.new_line()?;
        }

        Ok(())
    }

    pub fn cleanup(&mut self, message: &str, answer: &str) -> InquireResult<()> {
        self.reset_prompt()?;
        self.print_prompt_answer(message, answer)?;

        Ok(())
    }

    pub fn flush(&mut self) -> InquireResult<()> {
        self.terminal.flush()?;

        Ok(())
    }

    pub fn read_key(&mut self) -> InquireResult<Key> {
        self.terminal
            .read_key()
            .map(Key::from)
            .map_err(InquireError::from)
    }

    fn new_line(&mut self) -> InquireResult<()> {
        self.terminal.cursor_horizontal_reset()?;
        self.terminal.write("\n")?;
        self.cur_line = self.cur_line.saturating_add(1);

        Ok(())
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        let _ = self.terminal.cursor_show();
    }
}
