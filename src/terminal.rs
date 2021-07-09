use std::io::{stdin, stdout, Error, Read, Stdin, Stdout, Write};

use crate::error::{InquireError, InquireResult};
use termion::{
    color::{self, Color},
    cursor,
    event::Key,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
};

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub enum IO<'a> {
    Std {
        r: Keys<Stdin>,
        w: RawTerminal<Stdout>,
    },
    #[allow(unused)]
    Custom {
        r: Keys<&'a mut (dyn 'a + Read)>,
        w: &'a mut (dyn Write),
    },
}

pub struct Terminal<'a> {
    io: IO<'a>,
    dull: bool,
}

#[derive(Copy, Clone)]
pub enum Style {
    Bold,
    #[allow(unused)]
    Italic,
}

impl<'a> Terminal<'a> {
    /// # Errors
    ///
    /// Will return `std::io::Error` if it fails to get terminal size
    pub fn new() -> InquireResult<Self> {
        let raw_mode = stdout().into_raw_mode().map_err(|e| {
            if e.raw_os_error() == Some(25i32) {
                InquireError::NotTTY
            } else {
                InquireError::from(e)
            }
        });

        Ok(Self {
            io: IO::Std {
                r: stdin().keys(),
                w: raw_mode?,
            },
            dull: false,
        })
    }

    /// # Errors
    ///
    /// Will return `std::io::Error` if it fails to get terminal size
    #[cfg(test)]
    pub fn new_with_io<W: 'a + Write>(writer: &'a mut W, reader: &'a mut dyn Read) -> Self {
        Self {
            io: IO::Custom {
                r: reader.keys(),
                w: writer,
            },
            dull: false,
        }
    }

    #[allow(unused)]
    pub fn dull(mut self) -> Self {
        self.dull = true;

        self
    }

    pub fn cursor_up(&mut self) -> Result<(), Error> {
        write!(self.get_writer(), "{}", cursor::Up(1))
    }

    pub fn cursor_horizontal_reset(&mut self) -> Result<(), Error> {
        write!(self.get_writer(), "\x1b[0G")
    }

    /// # Errors
    ///
    /// Will return error when call to stdout().flush() fails
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.get_writer().flush()
    }

    /// # Errors
    ///
    /// Will never return Error for now
    pub fn read_key(&mut self) -> Result<Key, std::io::Error> {
        loop {
            match &mut self.io {
                IO::Std { r, w: _ } => {
                    if let Some(key) = r.next() {
                        return key;
                    }
                }
                IO::Custom { r, w: _ } => {
                    if let Some(key) = r.next() {
                        return key;
                    }
                }
            }
        }
    }

    pub fn write(&mut self, val: &str) -> Result<(), std::io::Error> {
        write!(self.get_writer(), "{}", val)
    }

    pub fn cursor_hide(&mut self) -> Result<(), std::io::Error> {
        write!(self.get_writer(), "{}", termion::cursor::Hide)
    }

    pub fn cursor_show(&mut self) -> Result<(), std::io::Error> {
        write!(self.get_writer(), "{}", termion::cursor::Show)
    }

    pub fn clear_current_line(&mut self) -> Result<(), std::io::Error> {
        write!(self.get_writer(), "{}", termion::clear::CurrentLine)
    }

    pub fn set_style(&mut self, style: Style) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        match style {
            Style::Bold => write!(self.get_writer(), "{}", termion::style::Bold),
            Style::Italic => write!(self.get_writer(), "{}", termion::style::Italic),
        }
    }

    #[allow(unused)]
    pub fn reset_style(&mut self) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        write!(self.get_writer(), "{}", termion::style::Reset)
    }

    pub fn set_bg_color<C: Color>(&mut self, color: C) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        write!(self.get_writer(), "{}", color::Bg(color))
    }

    #[allow(unused)]
    pub fn reset_bg_color(&mut self) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        write!(self.get_writer(), "{}", color::Bg(color::Reset))
    }

    pub fn set_fg_color<C: Color>(&mut self, color: C) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        write!(self.get_writer(), "{}", color::Fg(color))
    }

    #[allow(unused)]
    pub fn reset_fg_color(&mut self) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        write!(self.get_writer(), "{}", color::Fg(color::Reset))
    }

    pub fn get_writer(&mut self) -> &mut dyn Write {
        match &mut self.io {
            IO::Std { r: _, w } => w,
            IO::Custom { r: _, w } => w,
        }
    }
}

impl<'a> Drop for Terminal<'a> {
    fn drop(&mut self) {
        let _ = self.cursor_show();
        let _ = self.flush();
    }
}

#[cfg(test)]
mod test {

    use crate::terminal::Style;

    use super::Terminal;

    #[test]
    fn writer() {
        let mut write: Vec<u8> = Vec::new();
        let read: Vec<u8> = Vec::new();
        let mut read = read.as_slice();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read);

            terminal.write("testing ").unwrap();
            terminal.write("writing ").unwrap();
            terminal.flush().unwrap();
            terminal.write("wow").unwrap();
        }

        assert_eq!(
            format!("testing writing wow{}", termion::cursor::Show),
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn style_management() {
        let mut write: Vec<u8> = Vec::new();
        let read: Vec<u8> = Vec::new();
        let mut read = read.as_slice();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read);

            terminal.set_style(Style::Bold).unwrap();
            terminal.set_style(Style::Italic).unwrap();
            terminal.set_style(Style::Bold).unwrap();
            terminal.reset_style().unwrap();
        }

        assert_eq!(
            format!(
                "{}{}{}{}{}",
                termion::style::Bold,
                termion::style::Italic,
                termion::style::Bold,
                termion::style::Reset,
                termion::cursor::Show,
            ),
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn fg_color_management() {
        let mut write: Vec<u8> = Vec::new();
        let read: Vec<u8> = Vec::new();
        let mut read = read.as_slice();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read);

            terminal.set_fg_color(termion::color::Red).unwrap();
            terminal.reset_fg_color().unwrap();
            terminal.set_fg_color(termion::color::Black).unwrap();
            terminal.set_fg_color(termion::color::Green).unwrap();
        }

        assert_eq!(
            format!(
                "{}{}{}{}{}",
                termion::color::Fg(termion::color::Red),
                termion::color::Fg(termion::color::Reset),
                termion::color::Fg(termion::color::Black),
                termion::color::Fg(termion::color::Green),
                termion::cursor::Show,
            ),
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn bg_color_management() {
        let mut write: Vec<u8> = Vec::new();
        let read: Vec<u8> = Vec::new();
        let mut read = read.as_slice();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read);

            terminal.set_bg_color(termion::color::Red).unwrap();
            terminal.reset_bg_color().unwrap();
            terminal.set_bg_color(termion::color::Black).unwrap();
            terminal.set_bg_color(termion::color::Green).unwrap();
        }

        assert_eq!(
            format!(
                "{}{}{}{}{}",
                termion::color::Bg(termion::color::Red),
                termion::color::Bg(termion::color::Reset),
                termion::color::Bg(termion::color::Black),
                termion::color::Bg(termion::color::Green),
                termion::cursor::Show,
            ),
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn dull_ignores_fg_bg_style() {
        let mut write: Vec<u8> = Vec::new();
        let read: Vec<u8> = Vec::new();
        let mut read = read.as_slice();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read).dull();

            terminal.set_style(Style::Bold).unwrap();
            terminal.set_style(Style::Italic).unwrap();
            terminal.set_style(Style::Bold).unwrap();
            terminal.reset_style().unwrap();
            terminal.set_bg_color(termion::color::Red).unwrap();
            terminal.reset_bg_color().unwrap();
            terminal.set_bg_color(termion::color::Black).unwrap();
            terminal.set_bg_color(termion::color::Green).unwrap();
            terminal.write("wow").unwrap();
            terminal.set_fg_color(termion::color::Red).unwrap();
            terminal.reset_fg_color().unwrap();
            terminal.set_fg_color(termion::color::Black).unwrap();
            terminal.set_fg_color(termion::color::Green).unwrap();
        }

        assert_eq!(
            format!("wow{}", termion::cursor::Show),
            std::str::from_utf8(&write).unwrap()
        );
    }
}
