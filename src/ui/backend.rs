use std::{fmt::Display, io::Result};

use super::{Attributes, Color, Key, Styled};

pub trait Backend {
    fn cursor_up(&mut self) -> Result<()>;
    fn cursor_move_to_column(&mut self, idx: u16) -> Result<()>;
    fn read_key(&mut self) -> Result<Key>;
    fn flush(&mut self) -> Result<()>;

    fn write<T: Display>(&mut self, val: T) -> Result<()>;
    fn write_styled<T: Display>(&mut self, val: Styled<T>) -> Result<()>;

    fn clear_current_line(&mut self) -> Result<()>;

    fn cursor_hide(&mut self) -> Result<()>;
    fn cursor_show(&mut self) -> Result<()>;

    fn set_attributes(&mut self, attributes: Attributes) -> Result<()>;
    fn reset_attributes(&mut self) -> Result<()>;

    fn set_fg_color(&mut self, color: Color) -> Result<()>;
    fn reset_fg_color(&mut self) -> Result<()>;

    fn set_bg_color(&mut self, color: Color) -> Result<()>;
    fn reset_bg_color(&mut self) -> Result<()>;
}

pub mod crossterm {
    use std::io::{stdout, Result, Stdout, Write};

    use crossterm::{
        event::KeyEvent,
        queue,
        style::{Attribute, Print},
        terminal::{enable_raw_mode, ClearType},
    };

    use crate::{
        error::{InquireError, InquireResult},
        ui::{Attributes, Color, Key, Styled},
    };

    use super::Backend;

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

    pub struct CrosstermBackend<'a> {
        io: IO<'a>,
    }

    impl<'a> CrosstermBackend<'a> {
        pub fn new() -> InquireResult<Self> {
            enable_raw_mode().map_err(|e| {
                if e.raw_os_error() == Some(25i32) {
                    InquireError::NotTTY
                } else {
                    InquireError::from(e)
                }
            })?;

            Ok(Self {
                io: IO::Std { w: stdout() },
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
            }
        }

        fn get_writer(&mut self) -> &mut dyn Write {
            match &mut self.io {
                IO::Std { w } => w,
                IO::Custom { r: _, w } => w,
            }
        }
    }

    impl<'a> Backend for CrosstermBackend<'a> {
        fn cursor_up(&mut self) -> Result<()> {
            queue!(&mut self.get_writer(), crossterm::cursor::MoveUp(1))
        }

        fn cursor_move_to_column(&mut self, idx: u16) -> Result<()> {
            queue!(&mut self.get_writer(), crossterm::cursor::MoveToColumn(idx))
        }

        fn read_key(&mut self) -> Result<Key> {
            loop {
                match &mut self.io {
                    IO::Std { w: _ } => match crossterm::event::read()? {
                        crossterm::event::Event::Key(key_event) => return Ok(key_event.into()),
                        crossterm::event::Event::Mouse(_) => {}
                        crossterm::event::Event::Resize(_, _) => {}
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

        fn write<T: std::fmt::Display>(&mut self, val: T) -> Result<()> {
            queue!(&mut self.get_writer(), Print(val))
        }

        fn write_styled<T: std::fmt::Display>(&mut self, val: Styled<T>) -> Result<()> {
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

        fn clear_current_line(&mut self) -> Result<()> {
            queue!(
                &mut self.get_writer(),
                crossterm::terminal::Clear(ClearType::CurrentLine)
            )
        }

        fn cursor_hide(&mut self) -> Result<()> {
            queue!(&mut self.get_writer(), crossterm::cursor::Hide)
        }

        fn cursor_show(&mut self) -> Result<()> {
            queue!(&mut self.get_writer(), crossterm::cursor::Show)
        }

        fn set_attributes(&mut self, attributes: Attributes) -> Result<()> {
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

        fn reset_attributes(&mut self) -> Result<()> {
            queue!(
                &mut self.get_writer(),
                crossterm::style::SetAttribute(Attribute::Reset)
            )
        }

        fn set_fg_color(&mut self, color: Color) -> Result<()> {
            queue!(
                &mut self.get_writer(),
                crossterm::style::SetForegroundColor(color.into())
            )
        }

        fn reset_fg_color(&mut self) -> Result<()> {
            queue!(
                &mut self.get_writer(),
                crossterm::style::SetForegroundColor(crossterm::style::Color::Reset)
            )
        }

        fn set_bg_color(&mut self, color: Color) -> Result<()> {
            queue!(
                &mut self.get_writer(),
                crossterm::style::SetBackgroundColor(color.into())
            )
        }

        fn reset_bg_color(&mut self) -> Result<()> {
            queue!(
                &mut self.get_writer(),
                crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
            )
        }
    }

    impl<'a> Drop for CrosstermBackend<'a> {
        fn drop(&mut self) {
            let _ = self.flush();
            let _ = match self.io {
                IO::Std { w: _ } => crossterm::terminal::disable_raw_mode(),
                IO::Custom { r: _, w: _ } => Ok(()),
            };
        }
    }

    impl From<Color> for crossterm::style::Color {
        fn from(c: Color) -> Self {
            match c {
                Color::Black => crossterm::style::Color::Black,
                Color::DarkGrey => crossterm::style::Color::DarkGrey,
                Color::Red => crossterm::style::Color::Red,
                Color::DarkRed => crossterm::style::Color::DarkRed,
                Color::Green => crossterm::style::Color::Green,
                Color::DarkGreen => crossterm::style::Color::DarkGreen,
                Color::Yellow => crossterm::style::Color::Yellow,
                Color::DarkYellow => crossterm::style::Color::DarkYellow,
                Color::Blue => crossterm::style::Color::Blue,
                Color::DarkBlue => crossterm::style::Color::DarkBlue,
                Color::Magenta => crossterm::style::Color::Magenta,
                Color::DarkMagenta => crossterm::style::Color::DarkMagenta,
                Color::Cyan => crossterm::style::Color::Cyan,
                Color::DarkCyan => crossterm::style::Color::DarkCyan,
                Color::White => crossterm::style::Color::White,
                Color::Grey => crossterm::style::Color::Grey,
                Color::Rgb { r, g, b } => crossterm::style::Color::Rgb { r, g, b },
                Color::AnsiValue(b) => crossterm::style::Color::AnsiValue(b),
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::ui::Backend;
        use crate::ui::Color;

        use super::Attributes;
        use super::CrosstermBackend;

        #[test]
        fn writer() {
            let mut write: Vec<u8> = Vec::new();
            let read = Vec::new();
            let mut read = read.iter();

            {
                let mut backend = CrosstermBackend::new_with_io(&mut write, &mut read);

                backend.write("testing ").unwrap();
                backend.write("writing ").unwrap();
                backend.flush().unwrap();
                backend.write("wow").unwrap();
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
                let mut backend = CrosstermBackend::new_with_io(&mut write, &mut read);

                backend.set_attributes(Attributes::BOLD).unwrap();
                backend.set_attributes(Attributes::ITALIC).unwrap();
                backend.set_attributes(Attributes::BOLD).unwrap();
                backend.reset_attributes().unwrap();
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
                let mut backend = CrosstermBackend::new_with_io(&mut write, &mut read);

                backend
                    .set_attributes(Attributes::BOLD | Attributes::ITALIC | Attributes::BOLD)
                    .unwrap();
                backend.reset_attributes().unwrap();
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
                let mut backend = CrosstermBackend::new_with_io(&mut write, &mut read);

                backend.set_fg_color(Color::Red).unwrap();
                backend.reset_fg_color().unwrap();
                backend.set_fg_color(Color::Black).unwrap();
                backend.set_fg_color(Color::Green).unwrap();
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
                let mut backend = CrosstermBackend::new_with_io(&mut write, &mut read);

                backend.set_bg_color(Color::Red).unwrap();
                backend.reset_bg_color().unwrap();
                backend.set_bg_color(Color::Black).unwrap();
                backend.set_bg_color(Color::Green).unwrap();
            }

            #[cfg(unix)]
            assert_eq!(
                "\x1B[48;5;9m\x1B[49m\x1B[48;5;0m\x1B[48;5;10m",
                std::str::from_utf8(&write).unwrap()
            );
        }
    }
}
