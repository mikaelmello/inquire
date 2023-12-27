use std::io::{Result, Write};

use console::{Attribute, Color, Key, Style, Term};

use crate::ui::{Attributes, StyleSheet, Styled};

use super::{Terminal, INITIAL_IN_MEMORY_CAPACITY};

pub struct ConsoleTerminal {
    term: Term,
    in_memory_content: String,
}

impl ConsoleTerminal {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            term: Term::stderr(),
            in_memory_content: String::with_capacity(INITIAL_IN_MEMORY_CAPACITY),
        }
    }
}

impl Terminal for ConsoleTerminal {
    fn cursor_up(&mut self, cnt: u16) -> Result<()> {
        self.term.move_cursor_up(cnt as usize)
    }

    fn cursor_down(&mut self, cnt: u16) -> Result<()> {
        self.term.move_cursor_down(cnt as usize)
    }

    fn cursor_move_to_column(&mut self, idx: u16) -> Result<()> {
        // console has no built-in method to set cursor column ¯\_(ツ)_/¯
        self.term.move_cursor_left(1000)?;
        self.term.move_cursor_right(idx as usize)?;

        Ok(())
    }

    fn read_key(&mut self) -> Result<crate::ui::Key> {
        self.term.read_key().map(|k| k.into())
    }

    fn flush(&mut self) -> Result<()> {
        self.term.flush()
    }

    fn get_size(&self) -> Result<super::TerminalSize> {
        let (height, width) = self.term.size();

        Ok(super::TerminalSize { width, height })
    }

    fn write<T: std::fmt::Display>(&mut self, val: T) -> Result<()> {
        let formatted = format!("{}", val);
        let converted = newline_converter::unix2dos(&formatted);

        self.in_memory_content.push_str(converted.as_ref());
        write!(self.term, "{}", converted)
    }

    fn write_styled<T: std::fmt::Display>(&mut self, val: &Styled<T>) -> Result<()> {
        let formatted = format!("{}", val.content);
        let converted = newline_converter::unix2dos(&formatted);

        self.in_memory_content.push_str(converted.as_ref());

        let styled_object = Style::from(val.style).apply_to(converted);

        write!(self.term, "{}", styled_object)
    }

    fn clear_current_line(&mut self) -> Result<()> {
        self.term.clear_line()
    }

    fn cursor_hide(&mut self) -> Result<()> {
        self.term.hide_cursor()
    }

    fn cursor_show(&mut self) -> Result<()> {
        self.term.show_cursor()
    }

    fn get_in_memory_content(&self) -> &str {
        self.in_memory_content.as_ref()
    }

    fn clear_in_memory_content(&mut self) {
        self.in_memory_content.clear();
    }
}

impl Drop for ConsoleTerminal {
    fn drop(&mut self) {
        let _unused = self.flush();
    }
}

impl From<StyleSheet> for Style {
    fn from(from: StyleSheet) -> Self {
        let mut style = Style::new();

        let bg = from.bg.and_then(crate::ui::Color::into_console_color);
        if let Some(bg) = bg {
            style = style.bg(bg);
        }

        let fg = from.fg.and_then(crate::ui::Color::into_console_color);
        if let Some(fg) = fg {
            style = style.fg(fg);
        }

        if from.att.contains(Attributes::BOLD) {
            style = style.attr(Attribute::Bold);
        }

        if from.att.contains(Attributes::ITALIC) {
            style = style.attr(Attribute::Italic);
        }

        style
    }
}

impl crate::ui::Color {
    fn into_console_color(self) -> Option<Color> {
        use crate::ui::Color as C;
        match self {
            C::Black | C::DarkGrey => Some(Color::Black),
            C::LightRed | C::DarkRed => Some(Color::Red),
            C::LightGreen | C::DarkGreen => Some(Color::Green),
            C::LightYellow | C::DarkYellow => Some(Color::Yellow),
            C::LightBlue | C::DarkBlue => Some(Color::Blue),
            C::LightMagenta | C::DarkMagenta => Some(Color::Magenta),
            C::LightCyan | C::DarkCyan => Some(Color::Cyan),
            C::White | C::Grey => Some(Color::White),
            C::Rgb { r: _, g: _, b: _ } => None,
            C::AnsiValue(v) => Some(Color::Color256(v)),
        }
    }
}

impl From<Key> for crate::ui::Key {
    fn from(key: Key) -> Self {
        use crate::ui::KeyModifiers;

        match key {
            Key::Escape => Self::Escape,
            Key::Char('\n' | '\r') | Key::Enter => Self::Enter,
            Key::Char('\t') | Key::Tab => Self::Tab,
            Key::Backspace => Self::Backspace,
            Key::Del => Self::Delete(KeyModifiers::empty()),
            Key::Home => Self::Home,
            Key::End => Self::End,
            Key::PageUp => Self::PageUp(KeyModifiers::empty()),
            Key::PageDown => Self::PageDown(KeyModifiers::empty()),
            Key::ArrowUp => Self::Up(KeyModifiers::empty()),
            Key::ArrowDown => Self::Down(KeyModifiers::empty()),
            Key::ArrowLeft => Self::Left(KeyModifiers::empty()),
            Key::ArrowRight => Self::Right(KeyModifiers::empty()),
            Key::Char(c) => Self::Char(c, KeyModifiers::empty()),
            #[allow(deprecated)]
            _ => Self::Any,
        }
    }
}
