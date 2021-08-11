use crate::{
    error::{InquireError, InquireResult},
    ui::Attributes,
};
use crossterm::{
    event::KeyEvent,
    queue,
    style::{Attribute, Print},
    terminal::{self, ClearType},
};
use std::{
    fmt::Display,
    io::{stdout, Error, Stdout, Write},
};

use super::style::Styled;

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

pub struct OldTerminal<'a> {
    io: IO<'a>,
    dull: bool,
}

impl<'a> OldTerminal<'a> {
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

    pub fn write<T: Display>(&mut self, val: T) -> Result<(), std::io::Error> {
        queue!(&mut self.get_writer(), Print(val))
    }

    pub fn write_styled<T: Display>(&mut self, val: Styled<T>) -> Result<(), std::io::Error> {
        if let Some(color) = val.style.fg {
            self.set_fg_color(color)?;
        }
        if let Some(color) = val.style.bg {
            self.set_bg_color(color)?;
        }
        if !val.style.att.is_empty() {
            self.set_attributes(val.style.att)?;
        }

        self.write(val.content)?;

        if let Some(_) = val.style.fg.as_ref() {
            self.reset_fg_color()?;
        }
        if let Some(_) = val.style.bg.as_ref() {
            self.reset_bg_color()?;
        }
        if !val.style.att.is_empty() {
            self.reset_attributes()?;
        }

        Ok(())
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

    pub fn set_attributes(&mut self, attributes: Attributes) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        if attributes.contains(Attributes::BOLD) {
            queue!(
                &mut self.get_writer(),
                crossterm::style::SetAttribute(Attribute::Bold)
            )?;
        }
        if attributes.contains(Attributes::ITALIC) {
            queue!(
                &mut self.get_writer(),
                crossterm::style::SetAttribute(Attribute::Italic)
            )?;
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn reset_attributes(&mut self) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        queue!(
            &mut self.get_writer(),
            crossterm::style::SetAttribute(Attribute::Reset)
        )
    }

    pub fn set_bg_color<T: Into<crossterm::style::Color>>(
        &mut self,
        color: T,
    ) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        queue!(
            &mut self.get_writer(),
            crossterm::style::SetBackgroundColor(color.into())
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

    pub fn set_fg_color<T: Into<crossterm::style::Color>>(
        &mut self,
        color: T,
    ) -> Result<(), std::io::Error> {
        if self.dull {
            return Ok(());
        }

        queue!(
            &mut self.get_writer(),
            crossterm::style::SetForegroundColor(color.into())
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

impl<'a> Drop for OldTerminal<'a> {
    fn drop(&mut self) {
        let _ = self.cursor_show();
        let _ = self.flush();
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod test {
    use super::Attributes;
    use super::OldTerminal;

    #[test]
    fn writer() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = OldTerminal::new_with_io(&mut write, &mut read);

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
            let mut terminal = OldTerminal::new_with_io(&mut write, &mut read);

            terminal.set_attributes(Attributes::BOLD).unwrap();
            terminal.set_attributes(Attributes::ITALIC).unwrap();
            terminal.set_attributes(Attributes::BOLD).unwrap();
            terminal.reset_attributes().unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[1m\x1B[3m\x1B[1m\x1B[0m\x1B[?25h",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn style_management_with_flags() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = OldTerminal::new_with_io(&mut write, &mut read);

            terminal
                .set_attributes(Attributes::BOLD | Attributes::ITALIC | Attributes::BOLD)
                .unwrap();
            terminal.reset_attributes().unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[1m\x1B[3m\x1B[0m\x1B[?25h",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn fg_color_management() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = OldTerminal::new_with_io(&mut write, &mut read);

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
            let mut terminal = OldTerminal::new_with_io(&mut write, &mut read);

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
            let mut terminal = OldTerminal::new_with_io(&mut write, &mut read).dull();

            terminal
                .set_attributes(Attributes::BOLD | Attributes::ITALIC)
                .unwrap();
            terminal.reset_attributes().unwrap();
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
