use std::io::{stdout, Result, Stdout, Write};

use crossterm::{
    cursor,
    event::{self, KeyEvent},
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, enable_raw_mode, ClearType},
    Command,
};

use crate::{
    error::{InquireError, InquireResult},
    ui::{Attributes, Key, Styled},
};

use super::{Terminal, INITIAL_IN_MEMORY_CAPACITY};

enum IO<'a> {
    Std {
        w: Stdout,
    },
    #[allow(unused)]
    Custom {
        r: &'a mut dyn Iterator<Item = &'a KeyEvent>,
        w: &'a mut (dyn Write),
    },
}

pub struct CrosstermTerminal<'a> {
    io: IO<'a>,
    in_memory_content: String,
}

impl<'a> CrosstermTerminal<'a> {
    pub fn new() -> InquireResult<Self> {
        enable_raw_mode().map_err(|e| match e.raw_os_error() {
            Some(25) | Some(6) => InquireError::NotTTY,
            _ => InquireError::from(e),
        })?;

        Ok(Self {
            io: IO::Std { w: stdout() },
            in_memory_content: String::with_capacity(INITIAL_IN_MEMORY_CAPACITY),
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
            in_memory_content: String::with_capacity(INITIAL_IN_MEMORY_CAPACITY),
        }
    }

    fn get_writer(&mut self) -> &mut dyn Write {
        match &mut self.io {
            IO::Std { w } => w,
            IO::Custom { r: _, w } => w,
        }
    }

    fn write_command<C: Command>(&mut self, command: C) -> Result<()> {
        queue!(&mut self.get_writer(), command)
    }
}

impl<'a> Terminal for CrosstermTerminal<'a> {
    fn cursor_up(&mut self, cnt: u16) -> Result<()> {
        self.write_command(cursor::MoveUp(cnt))
    }

    fn cursor_down(&mut self, cnt: u16) -> Result<()> {
        self.write_command(cursor::MoveDown(cnt))
    }

    fn cursor_move_to_column(&mut self, idx: u16) -> Result<()> {
        self.write_command(cursor::MoveToColumn(idx.saturating_add(1)))
    }

    fn read_key(&mut self) -> Result<Key> {
        loop {
            match &mut self.io {
                IO::Std { w: _ } => match event::read()? {
                    event::Event::Key(key_event) => return Ok(key_event.into()),
                    event::Event::Mouse(_) => {}
                    event::Event::Resize(_, _) => {}
                },
                IO::Custom { r, w: _ } => {
                    let key = r.next().expect("Custom stream of characters has ended");
                    return Ok((*key).into());
                }
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.get_writer().flush()
    }

    fn get_size(&self) -> Result<super::TerminalSize> {
        terminal::size().map(|(width, height)| super::TerminalSize { width, height })
    }

    fn write<T: std::fmt::Display>(&mut self, val: T) -> Result<()> {
        let formatted = format!("{}", val);
        let converted = newline_converter::unix2dos(&formatted);

        self.in_memory_content.push_str(converted.as_ref());
        self.write_command(Print(converted))
    }

    fn write_styled<'s, T: std::fmt::Display>(&mut self, val: &'s Styled<T>) -> Result<()> {
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

    fn clear_current_line(&mut self) -> Result<()> {
        self.write_command(terminal::Clear(ClearType::CurrentLine))
    }

    fn cursor_hide(&mut self) -> Result<()> {
        self.write_command(cursor::Hide)
    }

    fn cursor_show(&mut self) -> Result<()> {
        self.write_command(cursor::Show)
    }

    fn set_attributes(&mut self, attributes: Attributes) -> Result<()> {
        if attributes.contains(Attributes::BOLD) {
            self.write_command(SetAttribute(Attribute::Bold))?;
        }
        if attributes.contains(Attributes::ITALIC) {
            self.write_command(SetAttribute(Attribute::Italic))?;
        }

        Ok(())
    }

    fn reset_attributes(&mut self) -> Result<()> {
        self.write_command(SetAttribute(Attribute::Reset))
    }

    fn set_fg_color(&mut self, color: crate::ui::Color) -> Result<()> {
        self.write_command(SetForegroundColor(color.into()))
    }

    fn reset_fg_color(&mut self) -> Result<()> {
        self.write_command(SetForegroundColor(Color::Reset))
    }

    fn set_bg_color(&mut self, color: crate::ui::Color) -> Result<()> {
        self.write_command(SetBackgroundColor(color.into()))
    }

    fn reset_bg_color(&mut self) -> Result<()> {
        self.write_command(SetBackgroundColor(Color::Reset))
    }

    fn get_in_memory_content(&self) -> &str {
        self.in_memory_content.as_ref()
    }

    fn clear_in_memory_content(&mut self) {
        self.in_memory_content.clear()
    }
}

impl<'a> Drop for CrosstermTerminal<'a> {
    fn drop(&mut self) {
        let _ = self.flush();
        let _ = match self.io {
            IO::Std { w: _ } => terminal::disable_raw_mode(),
            IO::Custom { r: _, w: _ } => Ok(()),
        };
    }
}

impl From<crate::ui::Color> for Color {
    fn from(c: crate::ui::Color) -> Self {
        match c {
            crate::ui::Color::Black => Color::Black,
            crate::ui::Color::DarkGrey => Color::DarkGrey,
            crate::ui::Color::Red => Color::Red,
            crate::ui::Color::DarkRed => Color::DarkRed,
            crate::ui::Color::Green => Color::Green,
            crate::ui::Color::DarkGreen => Color::DarkGreen,
            crate::ui::Color::Yellow => Color::Yellow,
            crate::ui::Color::DarkYellow => Color::DarkYellow,
            crate::ui::Color::Blue => Color::Blue,
            crate::ui::Color::DarkBlue => Color::DarkBlue,
            crate::ui::Color::Magenta => Color::Magenta,
            crate::ui::Color::DarkMagenta => Color::DarkMagenta,
            crate::ui::Color::Cyan => Color::Cyan,
            crate::ui::Color::DarkCyan => Color::DarkCyan,
            crate::ui::Color::White => Color::White,
            crate::ui::Color::Grey => Color::Grey,
            crate::ui::Color::Rgb { r, g, b } => Color::Rgb { r, g, b },
            crate::ui::Color::AnsiValue(b) => Color::AnsiValue(b),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::terminal::Terminal;
    use crate::ui::Color;

    use super::Attributes;
    use super::CrosstermTerminal;

    #[test]
    fn writer() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);

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
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);

            terminal.set_attributes(Attributes::BOLD).unwrap();
            terminal.set_attributes(Attributes::ITALIC).unwrap();
            terminal.set_attributes(Attributes::BOLD).unwrap();
            terminal.reset_attributes().unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[1m\x1B[3m\x1B[1m\x1B[0m",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn style_management_with_flags() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);

            terminal
                .set_attributes(Attributes::BOLD | Attributes::ITALIC | Attributes::BOLD)
                .unwrap();
            terminal.reset_attributes().unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[1m\x1B[3m\x1B[0m",
            std::str::from_utf8(&write).unwrap()
        );
    }

    #[test]
    fn fg_color_management() {
        let mut write: Vec<u8> = Vec::new();
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);

            terminal.set_fg_color(Color::Red).unwrap();
            terminal.reset_fg_color().unwrap();
            terminal.set_fg_color(Color::Black).unwrap();
            terminal.set_fg_color(Color::Green).unwrap();
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
        let read = Vec::new();
        let mut read = read.iter();

        {
            let mut terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);

            terminal.set_bg_color(Color::Red).unwrap();
            terminal.reset_bg_color().unwrap();
            terminal.set_bg_color(Color::Black).unwrap();
            terminal.set_bg_color(Color::Green).unwrap();
        }

        #[cfg(unix)]
        assert_eq!(
            "\x1B[48;5;9m\x1B[49m\x1B[48;5;0m\x1B[48;5;10m",
            std::str::from_utf8(&write).unwrap()
        );
    }
}
