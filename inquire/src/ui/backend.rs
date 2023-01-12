use std::{collections::BTreeSet, fmt::Display, io::Result};

use unicode_width::UnicodeWidthChar;

use crate::{
    input::Input,
    list_option::ListOption,
    terminal::{Terminal, TerminalSize},
    ui::{IndexPrefix, Key, RenderConfig, Styled},
    utils::{int_log10, Page},
    validator::ErrorMessage,
};

pub trait CommonBackend {
    fn read_key(&mut self) -> Result<Key>;

    fn frame_setup(&mut self) -> Result<()>;
    fn frame_finish(&mut self) -> Result<()>;

    fn render_canceled_prompt(&mut self, prompt: &str) -> Result<()>;
    fn render_prompt_with_answer(&mut self, prompt: &str, answer: &str) -> Result<()>;

    fn render_error_message(&mut self, error: &ErrorMessage) -> Result<()>;
    fn render_help_message(&mut self, help: &str) -> Result<()>;
}

pub trait TextBackend: CommonBackend {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> Result<()>;
    fn render_suggestions<D: Display>(&mut self, page: Page<ListOption<D>>) -> Result<()>;
}

#[cfg(feature = "editor")]
pub trait EditorBackend: CommonBackend {
    fn render_prompt(&mut self, prompt: &str, editor_command: &str) -> Result<()>;
}

pub trait SelectBackend: CommonBackend {
    fn render_select_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
    fn render_options<D: Display>(&mut self, page: Page<ListOption<D>>) -> Result<()>;
}

pub trait MultiSelectBackend: CommonBackend {
    fn render_multiselect_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
    fn render_options<D: Display>(
        &mut self,
        page: Page<ListOption<D>>,
        checked: &BTreeSet<usize>,
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
    fn render_prompt(&mut self, prompt: &str) -> Result<()>;
    fn render_prompt_with_masked_input(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
    fn render_prompt_with_full_input(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

pub struct Backend<T>
where
    T: Terminal,
{
    prompt_current_position: Position,
    prompt_end_position: Position,
    prompt_cursor_offset: Option<usize>,
    prompt_cursor_position: Option<Position>,
    show_cursor: bool,
    terminal: T,
    terminal_size: TerminalSize,
    render_config: RenderConfig,
}

impl<T> Backend<T>
where
    T: Terminal,
{
    pub fn new(terminal: T, render_config: RenderConfig) -> Result<Self> {
        let terminal_size = terminal.get_size().unwrap_or(TerminalSize {
            width: 1000,
            height: 1000,
        });

        let mut backend = Self {
            prompt_current_position: Position::default(),
            prompt_end_position: Position::default(),
            prompt_cursor_offset: None,
            prompt_cursor_position: None,
            show_cursor: false,
            terminal,
            render_config,
            terminal_size,
        };

        backend.terminal.cursor_hide()?;

        Ok(backend)
    }

    fn update_position_info(&mut self) {
        let input = self.terminal.get_in_memory_content();
        let term_width = self.terminal_size.width;

        let mut cur_pos = Position::default();

        for (idx, c) in input.chars().enumerate() {
            let len = UnicodeWidthChar::width(c).unwrap_or(0) as u16;

            if c == '\n' {
                cur_pos.row = cur_pos.row.saturating_add(1);
                cur_pos.col = 0;
            } else {
                let left = term_width - cur_pos.col;

                if left >= len {
                    cur_pos.col = cur_pos.col.saturating_add(len);
                } else {
                    cur_pos.row = cur_pos.row.saturating_add(1);
                    cur_pos.col = len;
                }
            }

            if let Some(prompt_cursor_offset) = self.prompt_cursor_offset {
                if prompt_cursor_offset == idx {
                    let mut cursor_position = cur_pos;
                    cursor_position.col = cursor_position.col.saturating_sub(len);
                    self.prompt_cursor_position = Some(cursor_position);
                }
            }
        }

        self.prompt_current_position = cur_pos;
        self.prompt_end_position = cur_pos;
    }

    fn move_cursor_to_end_position(&mut self) -> Result<()> {
        if self.prompt_current_position.row != self.prompt_end_position.row {
            let diff = self
                .prompt_end_position
                .row
                .saturating_sub(self.prompt_current_position.row);
            self.terminal.cursor_down(diff)?;
            self.terminal
                .cursor_move_to_column(self.prompt_end_position.col)?;
        }

        Ok(())
    }

    fn update_cursor_status(&mut self) -> Result<()> {
        match self.show_cursor {
            true => self.terminal.cursor_show(),
            false => self.terminal.cursor_hide(),
        }
    }

    fn mark_prompt_cursor_position(&mut self, offset: usize) {
        let current = self.terminal.get_in_memory_content();
        let position = current.chars().count();
        let position = position.saturating_add(offset);

        self.prompt_cursor_offset = Some(position);
    }

    fn reset_prompt(&mut self) -> Result<()> {
        self.move_cursor_to_end_position()?;

        for _ in 0..self.prompt_end_position.row {
            self.terminal.cursor_up(1)?;
            self.terminal.clear_current_line()?;
        }

        self.terminal.clear_in_memory_content();

        self.prompt_current_position = Position::default();
        self.prompt_end_position = Position::default();
        self.prompt_cursor_position = None;
        self.prompt_cursor_offset = None;

        // let's default to false to catch any previous
        // default behaviors we didn't account for
        self.show_cursor = false;
        self.terminal.cursor_hide()?;

        Ok(())
    }

    fn print_option_prefix<D: Display>(
        &mut self,
        idx: usize,
        page: &Page<ListOption<D>>,
    ) -> Result<()> {
        let empty_prefix = Styled::new(" ");

        let x = if idx == page.selection {
            self.render_config.highlighted_option_prefix
        } else if idx == 0 && !page.first {
            self.render_config.scroll_up_prefix
        } else if (idx + 1) == page.content.len() && !page.last {
            self.render_config.scroll_down_prefix
        } else {
            empty_prefix
        };

        self.terminal.write_styled(&x)
    }

    fn print_option_value<D: Display>(&mut self, option: &ListOption<D>) -> Result<()> {
        self.terminal
            .write_styled(&Styled::new(&option.value).with_style_sheet(self.render_config.option))
    }

    fn print_option_index_prefix(&mut self, index: usize, max_index: usize) -> Option<Result<()>> {
        let index = index.saturating_add(1);

        let content = match self.render_config.option_index_prefix {
            IndexPrefix::None => None,
            IndexPrefix::Simple => Some(format!("{})", index)),
            IndexPrefix::SpacePadded => {
                let width = int_log10(max_index.saturating_add(1));
                Some(format!("{:width$})", index, width = width))
            }
            IndexPrefix::ZeroPadded => {
                let width = int_log10(max_index.saturating_add(1));
                Some(format!("{:0width$})", index, width = width))
            }
        };

        content.map(|prefix| {
            self.terminal
                .write_styled(&Styled::new(prefix).with_style_sheet(self.render_config.option))
        })
    }

    fn print_default_value(&mut self, value: &str) -> Result<()> {
        let content = format!("({})", value);
        let token = Styled::new(content).with_style_sheet(self.render_config.default_value);

        self.terminal.write_styled(&token)
    }

    fn print_prompt_with_prefix(&mut self, prefix: Styled<&str>, prompt: &str) -> Result<()> {
        self.terminal.write_styled(&prefix)?;

        self.terminal.write(" ")?;

        self.terminal
            .write_styled(&Styled::new(prompt).with_style_sheet(self.render_config.prompt))?;

        Ok(())
    }

    fn print_prompt(&mut self, prompt: &str) -> Result<()> {
        self.print_prompt_with_prefix(self.render_config.prompt_prefix, prompt)
    }

    fn print_input(&mut self, input: &Input) -> Result<()> {
        self.terminal.write(" ")?;

        let cursor_offset = input.pre_cursor().chars().count();
        self.mark_prompt_cursor_position(cursor_offset);
        self.show_cursor = true;

        if input.is_empty() {
            match input.placeholder() {
                None => {}
                Some(p) if p.is_empty() => {}
                Some(p) => self.terminal.write_styled(
                    &Styled::new(p).with_style_sheet(self.render_config.placeholder),
                )?,
            }
        } else {
            self.terminal.write_styled(
                &Styled::new(input.content()).with_style_sheet(self.render_config.text_input),
            )?;
        }

        // if cursor is at end of input, we need to add
        // a space, otherwise the cursor will render on the
        // \n character, on the next line.
        if input.cursor() == input.length() {
            self.terminal.write(' ')?;
        }

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
            self.terminal.write(" ")?;
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
        self.terminal.write("\r\n")?;
        Ok(())
    }
}

impl<T> CommonBackend for Backend<T>
where
    T: Terminal,
{
    fn frame_setup(&mut self) -> Result<()> {
        self.terminal.cursor_hide()?;
        self.terminal.flush()?;

        self.reset_prompt()
    }

    fn frame_finish(&mut self) -> Result<()> {
        self.update_position_info();

        if let Some(prompt_cursor_position) = self.prompt_cursor_position {
            let row_diff = self.prompt_current_position.row - prompt_cursor_position.row;

            self.terminal.cursor_up(row_diff)?;
            self.terminal
                .cursor_move_to_column(prompt_cursor_position.col)?;

            self.prompt_current_position = prompt_cursor_position;
        }

        self.update_cursor_status()?;

        self.flush()
    }

    fn render_canceled_prompt(&mut self, prompt: &str) -> Result<()> {
        self.print_prompt(prompt)?;

        self.terminal.write(" ")?;

        self.terminal
            .write_styled(&self.render_config.canceled_prompt_indicator)?;

        self.new_line()?;

        Ok(())
    }

    fn render_prompt_with_answer(&mut self, prompt: &str, answer: &str) -> Result<()> {
        self.print_prompt_with_prefix(self.render_config.answered_prompt_prefix, prompt)?;

        self.terminal.write(" ")?;

        let token = Styled::new(answer).with_style_sheet(self.render_config.answer);
        self.terminal.write_styled(&token)?;

        self.new_line()?;

        Ok(())
    }

    fn read_key(&mut self) -> Result<Key> {
        self.terminal.read_key()
    }

    fn render_error_message(&mut self, error: &ErrorMessage) -> Result<()> {
        self.terminal
            .write_styled(&self.render_config.error_message.prefix)?;

        self.terminal.write_styled(
            &Styled::new(" ").with_style_sheet(self.render_config.error_message.separator),
        )?;

        let message = match error {
            ErrorMessage::Default => self.render_config.error_message.default_message,
            ErrorMessage::Custom(msg) => msg,
        };

        self.terminal.write_styled(
            &Styled::new(message).with_style_sheet(self.render_config.error_message.message),
        )?;

        self.new_line()?;

        Ok(())
    }

    fn render_help_message(&mut self, help: &str) -> Result<()> {
        self.terminal
            .write_styled(&Styled::new("[").with_style_sheet(self.render_config.help_message))?;

        self.terminal
            .write_styled(&Styled::new(help).with_style_sheet(self.render_config.help_message))?;

        self.terminal
            .write_styled(&Styled::new("]").with_style_sheet(self.render_config.help_message))?;

        self.new_line()?;

        Ok(())
    }
}

impl<T> TextBackend for Backend<T>
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

    fn render_suggestions<D: Display>(&mut self, page: Page<ListOption<D>>) -> Result<()> {
        for (idx, option) in page.content.iter().enumerate() {
            self.print_option_prefix(idx, &page)?;

            self.terminal.write(" ")?;

            self.print_option_value(option)?;

            self.new_line()?;
        }

        Ok(())
    }
}

#[cfg(feature = "editor")]
impl<T> EditorBackend for Backend<T>
where
    T: Terminal,
{
    fn render_prompt(&mut self, prompt: &str, editor_command: &str) -> Result<()> {
        self.print_prompt(prompt)?;

        self.terminal.write(" ")?;

        let message = format!("[(e) to open {}, (enter) to submit]", editor_command);
        let token = Styled::new(message).with_style_sheet(self.render_config.editor_prompt);
        self.terminal.write_styled(&token)?;

        self.new_line()?;

        Ok(())
    }
}

impl<T> SelectBackend for Backend<T>
where
    T: Terminal,
{
    fn render_select_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()> {
        self.print_prompt_with_input(prompt, None, cur_input)
    }

    fn render_options<D: Display>(&mut self, page: Page<ListOption<D>>) -> Result<()> {
        for (idx, option) in page.content.iter().enumerate() {
            self.print_option_prefix(idx, &page)?;

            self.terminal.write(" ")?;

            if let Some(res) = self.print_option_index_prefix(option.index, page.total) {
                res?;
                self.terminal.write(" ")?;
            }

            self.print_option_value(option)?;

            self.new_line()?;
        }

        Ok(())
    }
}

impl<T> MultiSelectBackend for Backend<T>
where
    T: Terminal,
{
    fn render_multiselect_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()> {
        self.print_prompt_with_input(prompt, None, cur_input)
    }

    fn render_options<D: Display>(
        &mut self,
        page: Page<ListOption<D>>,
        checked: &BTreeSet<usize>,
    ) -> Result<()> {
        for (idx, option) in page.content.iter().enumerate() {
            self.print_option_prefix(idx, &page)?;

            self.terminal.write(" ")?;

            if let Some(res) = self.print_option_index_prefix(option.index, page.total) {
                res?;
                self.terminal.write(" ")?;
            }

            match checked.contains(&option.index) {
                true => self
                    .terminal
                    .write_styled(&self.render_config.selected_checkbox)?,
                false => self
                    .terminal
                    .write_styled(&self.render_config.unselected_checkbox)?,
            }

            self.terminal.write(" ")?;

            self.print_option_value(option)?;

            self.new_line()?;
        }

        Ok(())
    }
}

#[cfg(feature = "date")]
pub mod date {
    use std::{io::Result, ops::Sub};

    use chrono::{Datelike, Duration};

    use crate::{date_utils::get_start_date, terminal::Terminal, ui::Styled};

    use super::{Backend, CommonBackend};

    pub trait DateSelectBackend: CommonBackend {
        fn render_calendar_prompt(&mut self, prompt: &str) -> Result<()>;

        #[allow(clippy::too_many_arguments)]
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

    impl<T> DateSelectBackend for Backend<T>
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
            macro_rules! write_prefix {
                () => {{
                    self.terminal
                        .write_styled(&self.render_config.calendar.prefix)?;
                    self.terminal.write(" ")
                }};
            }

            // print header (month year)
            let header = format!("{} {}", month.name().to_lowercase(), year);
            let header = format!("{:^20}", header);
            let header = Styled::new(header).with_style_sheet(self.render_config.calendar.header);

            write_prefix!()?;

            self.terminal.write_styled(&header)?;

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

            let week_days = Styled::new(week_days.join(" "))
                .with_style_sheet(self.render_config.calendar.week_header);

            write_prefix!()?;

            self.terminal.write_styled(&week_days)?;
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
                write_prefix!()?;

                for i in 0..7 {
                    if i > 0 {
                        self.terminal.write(" ")?;
                    }

                    let date = format!("{:2}", date_it.day());

                    let cursor_offset = if date_it.day() < 10 { 1 } else { 0 };

                    let mut style_sheet = crate::ui::StyleSheet::empty();

                    if date_it == selected_date {
                        self.mark_prompt_cursor_position(cursor_offset);
                        if let Some(custom_style_sheet) = self.render_config.calendar.selected_date
                        {
                            style_sheet = custom_style_sheet;
                        } else {
                            self.show_cursor = true;
                        }
                    } else if date_it == today {
                        style_sheet = self.render_config.calendar.today_date;
                    } else if date_it.month() != month.number_from_month() {
                        style_sheet = self.render_config.calendar.different_month_date;
                    }

                    if let Some(min_date) = min_date {
                        if date_it < min_date {
                            style_sheet = self.render_config.calendar.unavailable_date;
                        }
                    }

                    if let Some(max_date) = max_date {
                        if date_it > max_date {
                            style_sheet = self.render_config.calendar.unavailable_date;
                        }
                    }

                    let token = Styled::new(date).with_style_sheet(style_sheet);
                    self.terminal.write_styled(&token)?;

                    date_it = date_it.succ();
                }

                self.new_line()?;
            }

            Ok(())
        }
    }
}

impl<T> CustomTypeBackend for Backend<T>
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

impl<T> PasswordBackend for Backend<T>
where
    T: Terminal,
{
    fn render_prompt(&mut self, prompt: &str) -> Result<()> {
        self.print_prompt(prompt)?;
        self.new_line()?;
        Ok(())
    }

    fn render_prompt_with_masked_input(&mut self, prompt: &str, cur_input: &Input) -> Result<()> {
        let masked_string: String = (0..cur_input.length())
            .map(|_| self.render_config.password_mask)
            .collect();

        let masked_input = Input::new_with(&masked_string).with_cursor(cur_input.cursor());

        self.print_prompt_with_input(prompt, None, &masked_input)
    }

    fn render_prompt_with_full_input(&mut self, prompt: &str, cur_input: &Input) -> Result<()> {
        self.print_prompt_with_input(prompt, None, cur_input)
    }
}

impl<T> Drop for Backend<T>
where
    T: Terminal,
{
    fn drop(&mut self) {
        let _ = self.move_cursor_to_end_position();
        let _ = self.terminal.cursor_show();
    }
}
