use core::fmt;
use std::{
    fs::File,
    io::{Result, Write},
};

use termion::{
    color::{self, Color},
    cursor,
    event::Key,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

use crate::{
    error::InquireResult,
    ui::{Attributes, InputReader, Styled},
};

use super::Terminal;

#[allow(clippy::upper_case_acronyms)]
enum IO<'a> {
    TTY(RawTerminal<File>),
    #[allow(unused)]
    Custom(&'a mut (dyn Write)),
}

pub struct TermionKeyReader {
    keys: Keys<File>,
}

impl TermionKeyReader {
    #[allow(unused)]
    pub fn new() -> InquireResult<Self> {
        Ok(Self {
            keys: termion::get_tty()?.keys(),
        })
    }
}

impl InputReader for TermionKeyReader {
    fn read_key(&mut self) -> InquireResult<crate::ui::Key> {
        loop {
            if let Some(key) = self.keys.next() {
                let key = key?;
                return Ok(key.into());
            }
        }
    }
}

pub struct TermionTerminal<'a> {
    io: IO<'a>,
}

impl<'a> TermionTerminal<'a> {
    #[allow(unused)]
    pub fn new() -> InquireResult<Self> {
        let tty = termion::get_tty()?;
        let raw_terminal = tty.into_raw_mode()?;
        let keys = raw_terminal.try_clone()?.keys();

        Ok(Self {
            io: IO::TTY(raw_terminal),
        })
    }

    /// # Errors
    ///
    /// Will return `std::io::Error` if it fails to get terminal size
    #[cfg(test)]
    pub fn new_with_writer<W: 'a + Write>(writer: &'a mut W) -> Self {
        Self {
            io: IO::Custom(writer),
        }
    }

    fn get_writer(&mut self) -> &mut dyn Write {
        match &mut self.io {
            IO::TTY(w) => w,
            IO::Custom(w) => w,
        }
    }

    fn set_attributes(&mut self, attributes: Attributes) -> Result<()> {
        if attributes.contains(Attributes::BOLD) {
            write!(self.get_writer(), "{}", termion::style::Bold)?;
        }
        if attributes.contains(Attributes::ITALIC) {
            write!(self.get_writer(), "{}", termion::style::Italic)?;
        }

        Ok(())
    }

    fn reset_attributes(&mut self) -> Result<()> {
        write!(self.get_writer(), "{}", termion::style::Reset)
    }

    fn set_fg_color(&mut self, color: crate::ui::Color) -> Result<()> {
        write!(self.get_writer(), "{}", color::Fg(color))
    }

    fn reset_fg_color(&mut self) -> Result<()> {
        write!(self.get_writer(), "{}", color::Fg(color::Reset))
    }

    fn set_bg_color(&mut self, color: crate::ui::Color) -> Result<()> {
        write!(self.get_writer(), "{}", color::Bg(color))
    }

    fn reset_bg_color(&mut self) -> Result<()> {
        write!(self.get_writer(), "{}", color::Bg(color::Reset))
    }
}

impl<'a> Terminal for TermionTerminal<'a> {
    fn cursor_up(&mut self, cnt: u16) -> Result<()> {
        match cnt {
            0 => Ok(()),
            cnt => write!(self.get_writer(), "{}", cursor::Up(cnt)),
        }
    }

    fn cursor_down(&mut self, cnt: u16) -> Result<()> {
        match cnt {
            0 => Ok(()),
            cnt => write!(self.get_writer(), "{}", cursor::Down(cnt)),
        }
    }

    fn cursor_left(&mut self, cnt: u16) -> Result<()> {
        match cnt {
            0 => Ok(()),
            cnt => write!(self.get_writer(), "{}", cursor::Left(cnt)),
        }
    }

    fn cursor_right(&mut self, cnt: u16) -> Result<()> {
        match cnt {
            0 => Ok(()),
            cnt => write!(self.get_writer(), "{}", cursor::Right(cnt)),
        }
    }

    fn cursor_move_to_column(&mut self, idx: u16) -> Result<()> {
        write!(self.get_writer(), "\x1b[{}G", idx.saturating_add(1))
    }

    fn flush(&mut self) -> Result<()> {
        self.get_writer().flush()
    }

    fn get_size(&self) -> Result<super::TerminalSize> {
        terminal_size().map(|(width, height)| super::TerminalSize::new(width, height))
    }

    fn write<T: std::fmt::Display>(&mut self, val: T) -> Result<()> {
        write!(self.get_writer(), "{}", val)
    }

    fn write_styled<T: std::fmt::Display>(&mut self, val: &Styled<T>) -> Result<()> {
        if let Some(color) = val.style.fg {
            self.set_fg_color(color)?;
        }
        if let Some(color) = val.style.bg {
            self.set_bg_color(color)?;
        }
        if !val.style.att.is_empty() {
            self.set_attributes(val.style.att)?;
        }

        self.write(&val.content)?;

        if val.style.fg.as_ref().is_some() {
            self.reset_fg_color()?;
        }
        if val.style.bg.as_ref().is_some() {
            self.reset_bg_color()?;
        }
        if !val.style.att.is_empty() {
            self.reset_attributes()?;
        }

        Ok(())
    }

    fn clear_line(&mut self) -> Result<()> {
        write!(self.get_writer(), "{}", termion::clear::CurrentLine)
    }

    fn clear_until_new_line(&mut self) -> Result<()> {
        write!(self.get_writer(), "{}", termion::clear::UntilNewline)
    }

    fn cursor_hide(&mut self) -> Result<()> {
        write!(self.get_writer(), "{}", cursor::Hide)
    }

    fn cursor_show(&mut self) -> Result<()> {
        write!(self.get_writer(), "{}", cursor::Show)
    }
}

impl<'a> Drop for TermionTerminal<'a> {
    fn drop(&mut self) {
        let _unused = self.flush();
    }
}

macro_rules! into_termion_color {
    ($self:expr, $fn_name:ident, $f:expr) => {{
        use $crate::ui::Color as C;
        match $self {
            C::Black => color::Black.$fn_name($f),
            C::LightRed => color::LightRed.$fn_name($f),
            C::DarkRed => color::Red.$fn_name($f),
            C::LightGreen => color::LightGreen.$fn_name($f),
            C::DarkGreen => color::Green.$fn_name($f),
            C::LightYellow => color::LightYellow.$fn_name($f),
            C::DarkYellow => color::Yellow.$fn_name($f),
            C::LightBlue => color::LightBlue.$fn_name($f),
            C::DarkBlue => color::Blue.$fn_name($f),
            C::LightMagenta => color::LightMagenta.$fn_name($f),
            C::DarkMagenta => color::Magenta.$fn_name($f),
            C::LightCyan => color::LightCyan.$fn_name($f),
            C::DarkCyan => color::Cyan.$fn_name($f),
            C::White => color::LightWhite.$fn_name($f),
            C::Grey => color::White.$fn_name($f),
            C::DarkGrey => color::LightBlack.$fn_name($f),
            C::Rgb { r, g, b } => color::Rgb(*r, *g, *b).$fn_name($f),
            C::AnsiValue(b) => color::AnsiValue(*b).$fn_name($f),
        }
    }};
}

impl Color for crate::ui::Color {
    fn write_fg(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        into_termion_color!(self, write_fg, f)
    }

    fn write_bg(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        into_termion_color!(self, write_bg, f)
    }
}

impl From<Key> for crate::ui::Key {
    fn from(key: Key) -> Self {
        use crate::ui::KeyModifiers;

        match key {
            Key::Esc => Self::Escape,
            Key::Char('\n' | '\r') => Self::Enter,
            Key::Char('\t') => Self::Tab,
            Key::Backspace => Self::Backspace,
            Key::Delete => Self::Delete(KeyModifiers::empty()),
            Key::Home => Self::Home,
            Key::End => Self::End,
            Key::PageUp => Self::PageUp(KeyModifiers::empty()),
            Key::PageDown => Self::PageDown(KeyModifiers::empty()),
            Key::Up => Self::Up(KeyModifiers::empty()),
            Key::Down => Self::Down(KeyModifiers::empty()),
            Key::Left => Self::Left(KeyModifiers::empty()),
            Key::Right => Self::Right(KeyModifiers::empty()),
            Key::Char(c) => Self::Char(c, KeyModifiers::empty()),
            Key::Ctrl(c) => Self::Char(c, KeyModifiers::CONTROL),
            Key::Alt(c) => Self::Char(c, KeyModifiers::ALT),
            #[allow(deprecated)]
            _ => Self::Any,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::terminal::Terminal;
    use crate::ui::Color;

    use super::Attributes;
    use super::TermionTerminal;

    #[test]
    fn writer() {
        let mut write: Vec<u8> = Vec::new();

        {
            let mut terminal = TermionTerminal::new_with_writer(&mut write);

            terminal.write("testing ").unwrap();
            terminal.write("writing ").unwrap();
            terminal.flush().unwrap();
            terminal.write("wow").unwrap();
        }

        #[cfg(unix)]
        assert_eq!("testing writing wow", std::str::from_utf8(&write).unwrap());
    }

    #[test]
    fn style_management() {
        let mut write: Vec<u8> = Vec::new();

        {
            let mut terminal = TermionTerminal::new_with_writer(&mut write);

            terminal.set_attributes(Attributes::BOLD).unwrap();
            terminal.set_attributes(Attributes::ITALIC).unwrap();
            terminal.set_attributes(Attributes::BOLD).unwrap();
            terminal.reset_attributes().unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[1m\x1B[3m\x1B[1m\x1B[m",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn style_management_with_flags() {
        let mut write: Vec<u8> = Vec::new();

        {
            let mut terminal = TermionTerminal::new_with_writer(&mut write);

            terminal
                .set_attributes(Attributes::BOLD | Attributes::ITALIC | Attributes::BOLD)
                .unwrap();
            terminal.reset_attributes().unwrap();
        }

        #[cfg(unix)]
        assert_eq!("\x1B[1m\x1B[3m\x1B[m", std::str::from_utf8(&write).unwrap());
    }

    #[test]
    fn fg_color_management() {
        let mut write: Vec<u8> = Vec::new();

        {
            let mut terminal = TermionTerminal::new_with_writer(&mut write);

            terminal.set_fg_color(Color::LightRed).unwrap();
            terminal.reset_fg_color().unwrap();
            terminal.set_fg_color(Color::Black).unwrap();
            terminal.set_fg_color(Color::LightGreen).unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[38;5;9m\x1B[39m\x1B[38;5;0m\x1B[38;5;10m",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn bg_color_management() {
        let mut write: Vec<u8> = Vec::new();

        {
            let mut terminal = TermionTerminal::new_with_writer(&mut write);

            terminal.set_bg_color(Color::LightRed).unwrap();
            terminal.reset_bg_color().unwrap();
            terminal.set_bg_color(Color::Black).unwrap();
            terminal.set_bg_color(Color::LightGreen).unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[48;5;9m\x1B[49m\x1B[48;5;0m\x1B[48;5;10m",
            std::str::from_utf8(&write).unwrap()
        );
    }
}
