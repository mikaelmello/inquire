use std::io::{stdout, Error, Stdout, Write};

use crate::error::{InquireError, InquireResult};
use crossterm::{
    event::KeyEvent,
    queue,
    style::Attribute,
    terminal::{self, ClearType},
};

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub enum IO<'a> {
    Std {
        w: Stdout,
    },
    #[allow(unused)]
    Custom {
        r: &'a mut dyn Iterator<Item = &'a KeyEvent>,
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
        terminal::enable_raw_mode().map_err(|e| {
            if e.raw_os_error() == Some(25i32) {
                InquireError::NotTTY
            } else {
                InquireError::from(e)
            }
        })?;

        Ok(Self {
            io: IO::Std { w: stdout() },
            dull: false,
        })
    }

    /// # Errors
    ///
    /// Will return `std::io::Error` if it fails to get terminal size
    #[cfg(test)]
    pub fn new_with_io<W: 'a + Write>(
        writer: &'a mut W,
        reader: &'a mut dyn Iterator<Item = &'a KeyEvent>,
    ) -> Self {
        Self {
            io: IO::Custom {
                r: reader,
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
        queue!(&mut self.get_writer(), crossterm::cursor::MoveUp(1))
    }

    pub fn cursor_horizontal_reset(&mut self) -> Result<(), Error> {
        queue!(&mut self.get_writer(), crossterm::cursor::MoveToColumn(0))
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
    pub fn read_key(&mut self) -> Result<KeyEvent, std::io::Error> {
        loop {
            match &mut self.io {
                IO::Std { w: _ } => match crossterm::event::read()? {
                    crossterm::event::Event::Key(key_event) => return Ok(key_event),
                    crossterm::event::Event::Mouse(_) => {}
                    crossterm::event::Event::Resize(_, _) => {}
                },
                IO::Custom { r, w: _ } => {
                    let key = r.next().expect("Custom stream of characters has ended");
                    return Ok(key.clone());
                }
            }
        }
    }

    pub fn write(&mut self, val: &str) -> Result<(), std::io::Error> {
        write!(self.get_writer(), "{}", val)
    }

    pub fn cursor_hide(&mut self) -> Result<(), std::io::Error> {
        queue!(&mut self.get_writer(), crossterm::cursor::Hide)
    }

    pub fn cursor_show(&mut self) -> Result<(), std::io::Error> {
        queue!(&mut self.get_writer(), crossterm::cursor::Show)
    }

    pub fn clear_current_line(&mut self) -> Result<(), std::io::Error> {
        queue!(
            &mut self.get_writer(),
            crossterm::terminal::Clear(ClearType::CurrentLine)
        )
    }

    pub fn set_style(&mut self, style: Style) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        match style {
            Style::Bold => queue!(
                &mut self.get_writer(),
                crossterm::style::SetAttribute(Attribute::Bold)
            ),
            Style::Italic => {
                queue!(
                    &mut self.get_writer(),
                    crossterm::style::SetAttribute(Attribute::Italic)
                )
            }
        }
    }

    #[allow(unused)]
    pub fn reset_style(&mut self) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        queue!(
            &mut self.get_writer(),
            crossterm::style::SetAttribute(Attribute::Reset)
        )
    }

    pub fn set_bg_color(&mut self, color: crossterm::style::Color) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        queue!(
            &mut self.get_writer(),
            crossterm::style::SetBackgroundColor(color)
        )
    }

    #[allow(unused)]
    pub fn reset_bg_color(&mut self) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        queue!(
            &mut self.get_writer(),
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
        )
    }

    pub fn set_fg_color(&mut self, color: crossterm::style::Color) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        queue!(
            &mut self.get_writer(),
            crossterm::style::SetForegroundColor(color)
        )
    }

    #[allow(unused)]
    pub fn reset_fg_color(&mut self) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        queue!(
            &mut self.get_writer(),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Reset)
        )
    }

    pub fn get_writer(&mut self) -> &mut dyn Write {
        match &mut self.io {
            IO::Std { w } => w,
            IO::Custom { r: _, w } => w,
        }
    }
}

impl<'a> Drop for Terminal<'a> {
    fn drop(&mut self) {
        let _ = self.cursor_show();
        let _ = self.flush();
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod test {

    use crate::terminal::Style;

    use super::Terminal;

    #[test]
    fn writer() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read);

            terminal.write("testing ").unwrap();
            terminal.write("writing ").unwrap();
            terminal.flush().unwrap();
            terminal.write("wow").unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "testing writing wow\x1B[?25h",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn style_management() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read);

            terminal.set_style(Style::Bold).unwrap();
            terminal.set_style(Style::Italic).unwrap();
            terminal.set_style(Style::Bold).unwrap();
            terminal.reset_style().unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[1m\x1B[3m\x1B[1m\x1B[0m\x1B[?25h",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn fg_color_management() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read);

            terminal.set_fg_color(crossterm::style::Color::Red).unwrap();
            terminal.reset_fg_color().unwrap();
            terminal
                .set_fg_color(crossterm::style::Color::Black)
                .unwrap();
            terminal
                .set_fg_color(crossterm::style::Color::Green)
                .unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[38;5;9m\x1B[39m\x1B[38;5;0m\x1B[38;5;10m\x1B[?25h",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn bg_color_management() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read);

            terminal.set_bg_color(crossterm::style::Color::Red).unwrap();
            terminal.reset_bg_color().unwrap();
            terminal
                .set_bg_color(crossterm::style::Color::Black)
                .unwrap();
            terminal
                .set_bg_color(crossterm::style::Color::Green)
                .unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[48;5;9m\x1B[49m\x1B[48;5;0m\x1B[48;5;10m\x1B[?25h",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn dull_ignores_fg_bg_style() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = Terminal::new_with_io(&mut write, &mut read).dull();

            terminal.set_style(Style::Bold).unwrap();
            terminal.set_style(Style::Italic).unwrap();
            terminal.set_style(Style::Bold).unwrap();
            terminal.reset_style().unwrap();
            terminal.set_bg_color(crossterm::style::Color::Red).unwrap();
            terminal.reset_bg_color().unwrap();
            terminal
                .set_bg_color(crossterm::style::Color::Black)
                .unwrap();
            terminal
                .set_bg_color(crossterm::style::Color::Green)
                .unwrap();
            terminal.write("wow").unwrap();
            terminal.set_fg_color(crossterm::style::Color::Red).unwrap();
            terminal.reset_fg_color().unwrap();
            terminal
                .set_fg_color(crossterm::style::Color::Black)
                .unwrap();
            terminal
                .set_fg_color(crossterm::style::Color::Green)
                .unwrap();
        }

        #[cfg(unix)]
        assert_eq!("wow\x1B[?25h", std::str::from_utf8(&write).unwrap());
    }
}
