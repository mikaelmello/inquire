use std::{
    collections::VecDeque,
    io::{stderr, Result, Stderr, Write},
};

use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    Command,
};

use crate::{
    error::InquireResult,
    ui::{Attributes, Key, Styled},
};

use super::{Terminal, INITIAL_IN_MEMORY_CAPACITY};

enum IO {
    Std {
        w: Stderr,
    },
    #[allow(unused)]
    Test {
        w: Vec<u8>,
        r: VecDeque<KeyEvent>,
    },
}

pub struct CrosstermTerminal {
    io: IO,
    in_memory_content: String,
}

impl CrosstermTerminal {
    pub fn new() -> InquireResult<Self> {
        crossterm::terminal::enable_raw_mode()?;

        Ok(Self {
            io: IO::Std { w: stderr() },
            in_memory_content: String::with_capacity(INITIAL_IN_MEMORY_CAPACITY),
        })
    }

    fn get_writer(&mut self) -> &mut dyn Write {
        match &mut self.io {
            IO::Std { w } => w,
            IO::Test { r: _, w } => w,
        }
    }

    fn write_command<C: Command>(&mut self, command: C) -> Result<()> {
        queue!(&mut self.get_writer(), command)
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
}

impl Terminal for CrosstermTerminal {
    fn cursor_up(&mut self, cnt: u16) -> Result<()> {
        self.write_command(cursor::MoveUp(cnt))
    }

    fn cursor_down(&mut self, cnt: u16) -> Result<()> {
        self.write_command(cursor::MoveDown(cnt))
    }

    fn cursor_move_to_column(&mut self, idx: u16) -> Result<()> {
        self.write_command(cursor::MoveToColumn(idx))
    }

    fn read_key(&mut self) -> Result<Key> {
        loop {
            match &mut self.io {
                IO::Std { w: _ } => {
                    if let event::Event::Key(key_event) = event::read()? {
                        return Ok(key_event.into());
                    }
                }
                IO::Test { r, w: _ } => {
                    let key = r
                        .pop_front()
                        .expect("Custom stream of characters has ended");
                    return Ok(key.into());
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
        let formatted = format!("{val}");
        let converted = newline_converter::unix2dos(&formatted);

        self.in_memory_content.push_str(converted.as_ref());
        self.write_command(Print(converted))
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

    fn clear_current_line(&mut self) -> Result<()> {
        self.write_command(terminal::Clear(ClearType::CurrentLine))
    }

    fn cursor_hide(&mut self) -> Result<()> {
        self.write_command(cursor::Hide)
    }

    fn cursor_show(&mut self) -> Result<()> {
        self.write_command(cursor::Show)
    }

    fn get_in_memory_content(&self) -> &str {
        self.in_memory_content.as_ref()
    }

    fn clear_in_memory_content(&mut self) {
        self.in_memory_content.clear();
    }
}

impl Drop for CrosstermTerminal {
    fn drop(&mut self) {
        let _unused = self.flush();
        let _unused = match self.io {
            IO::Std { w: _ } => terminal::disable_raw_mode(),
            IO::Test { r: _, w: _ } => Ok(()),
        };
    }
}

impl From<crate::ui::Color> for Color {
    fn from(c: crate::ui::Color) -> Self {
        use crate::ui::Color as C;
        match c {
            C::Black => Color::Black,
            C::LightRed => Color::Red,
            C::DarkRed => Color::DarkRed,
            C::LightGreen => Color::Green,
            C::DarkGreen => Color::DarkGreen,
            C::LightYellow => Color::Yellow,
            C::DarkYellow => Color::DarkYellow,
            C::LightBlue => Color::Blue,
            C::DarkBlue => Color::DarkBlue,
            C::LightMagenta => Color::Magenta,
            C::DarkMagenta => Color::DarkMagenta,
            C::LightCyan => Color::Cyan,
            C::DarkCyan => Color::DarkCyan,
            C::White => Color::White,
            C::Grey => Color::Grey,
            C::DarkGrey => Color::DarkGrey,
            C::Rgb { r, g, b } => Color::Rgb { r, g, b },
            C::AnsiValue(b) => Color::AnsiValue(b),
        }
    }
}

impl From<KeyModifiers> for crate::ui::KeyModifiers {
    fn from(m: KeyModifiers) -> Self {
        let mut modifiers = Self::empty();

        if m.contains(KeyModifiers::NONE) {
            modifiers |= crate::ui::KeyModifiers::NONE;
        }
        if m.contains(KeyModifiers::ALT) {
            modifiers |= crate::ui::KeyModifiers::ALT;
        }
        if m.contains(KeyModifiers::CONTROL) {
            modifiers |= crate::ui::KeyModifiers::CONTROL;
        }
        if m.contains(KeyModifiers::SHIFT) {
            modifiers |= crate::ui::KeyModifiers::SHIFT;
        }

        modifiers
    }
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        match event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => Self::Escape,
            KeyEvent {
                code: KeyCode::Enter | KeyCode::Char('\n' | '\r'),
                ..
            } => Self::Enter,
            KeyEvent {
                code: KeyCode::Tab | KeyCode::Char('\t'),
                ..
            } => Self::Tab,
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => Self::Backspace,
            KeyEvent {
                code: KeyCode::Delete,
                modifiers: m,
                ..
            } => Self::Delete(m.into()),
            KeyEvent {
                code: KeyCode::Home,
                ..
            } => Self::Home,
            KeyEvent {
                code: KeyCode::End, ..
            } => Self::End,
            KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => Self::PageUp,
            KeyEvent {
                code: KeyCode::PageDown,
                ..
            } => Self::PageDown,
            KeyEvent {
                code: KeyCode::Up,
                modifiers: m,
                ..
            } => Self::Up(m.into()),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: m,
                ..
            } => Self::Down(m.into()),
            KeyEvent {
                code: KeyCode::Left,
                modifiers: m,
                ..
            } => Self::Left(m.into()),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: m,
                ..
            } => Self::Right(m.into()),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: m,
                ..
            } => Self::Char(c, m.into()),
            #[allow(deprecated)]
            _ => Self::Any,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use std::convert::TryFrom;
    use std::convert::TryInto;

    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    use crate::terminal::Terminal;
    use crate::terminal::INITIAL_IN_MEMORY_CAPACITY;
    use crate::ui::Color;
    use crate::ui::Key;

    use super::Attributes;
    use super::CrosstermTerminal;
    use super::IO;

    impl TryFrom<crate::ui::KeyModifiers> for KeyModifiers {
        type Error = ();

        fn try_from(value: crate::ui::KeyModifiers) -> Result<Self, Self::Error> {
            Self::from_bits(value.bits()).ok_or(())
        }
    }

    impl TryFrom<Key> for KeyEvent {
        type Error = ();

        fn try_from(value: Key) -> Result<Self, Self::Error> {
            let key_event = match value {
                Key::Escape => KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                Key::Enter => KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                Key::Backspace => KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                Key::Tab => KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                Key::Delete(m) => KeyEvent::new(KeyCode::Delete, m.try_into()?),
                Key::Home => KeyEvent::new(KeyCode::Home, KeyModifiers::NONE),
                Key::End => KeyEvent::new(KeyCode::End, KeyModifiers::NONE),
                Key::PageUp => KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
                Key::PageDown => KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
                Key::Up(m) => KeyEvent::new(KeyCode::Up, m.try_into()?),
                Key::Down(m) => KeyEvent::new(KeyCode::Down, m.try_into()?),
                Key::Left(m) => KeyEvent::new(KeyCode::Left, m.try_into()?),
                Key::Right(m) => KeyEvent::new(KeyCode::Right, m.try_into()?),
                Key::Char(c, m) => KeyEvent::new(KeyCode::Char(c), m.try_into()?),
                #[allow(deprecated)]
                Key::Any => KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            };

            Ok(key_event)
        }
    }

    impl CrosstermTerminal {
        pub fn new_with_io(events: VecDeque<KeyEvent>) -> Self {
            Self {
                io: IO::Test {
                    r: events,
                    w: Vec::new(),
                },
                in_memory_content: String::with_capacity(INITIAL_IN_MEMORY_CAPACITY),
            }
        }

        pub fn get_buffer_content(&mut self) -> Vec<u8> {
            match &mut self.io {
                IO::Std { w: _ } => panic!("Cannot get write buffer from standard output"),
                IO::Test { r: _, w } => {
                    let mut buffer = Vec::new();
                    std::mem::swap(&mut buffer, w);
                    buffer
                }
            }
        }
    }

    #[test]
    fn writer() {
        let mut terminal = CrosstermTerminal::new_with_io(VecDeque::new());

        terminal.write("testing ").unwrap();
        terminal.write("writing ").unwrap();
        terminal.flush().unwrap();
        terminal.write("wow").unwrap();

        #[cfg(unix)]
        assert_eq!(
            "testing writing wow",
            std::str::from_utf8(&terminal.get_buffer_content()).unwrap()
        );
    }

    #[test]
    fn style_management() {
        let mut terminal = CrosstermTerminal::new_with_io(VecDeque::new());

        terminal.set_attributes(Attributes::BOLD).unwrap();
        terminal.set_attributes(Attributes::ITALIC).unwrap();
        terminal.set_attributes(Attributes::BOLD).unwrap();
        terminal.reset_attributes().unwrap();

        #[cfg(unix)]
        assert_eq!(
            "\x1B[1m\x1B[3m\x1B[1m\x1B[0m",
            std::str::from_utf8(&terminal.get_buffer_content()).unwrap()
        );
    }

    #[test]
    fn style_management_with_flags() {
        let mut terminal = CrosstermTerminal::new_with_io(VecDeque::new());

        terminal
            .set_attributes(Attributes::BOLD | Attributes::ITALIC | Attributes::BOLD)
            .unwrap();
        terminal.reset_attributes().unwrap();

        #[cfg(unix)]
        assert_eq!(
            "\x1B[1m\x1B[3m\x1B[0m",
            std::str::from_utf8(&terminal.get_buffer_content()).unwrap()
        );
    }

    #[test]
    fn fg_color_management() {
        let mut terminal = CrosstermTerminal::new_with_io(VecDeque::new());

        terminal.set_fg_color(Color::LightRed).unwrap();
        terminal.reset_fg_color().unwrap();
        terminal.set_fg_color(Color::Black).unwrap();
        terminal.set_fg_color(Color::LightGreen).unwrap();

        #[cfg(unix)]
        assert_eq!(
            "\x1B[38;5;9m\x1B[39m\x1B[38;5;0m\x1B[38;5;10m",
            std::str::from_utf8(&terminal.get_buffer_content()).unwrap()
        );
    }

    #[test]
    fn bg_color_management() {
        let mut terminal = CrosstermTerminal::new_with_io(VecDeque::new());

        terminal.set_bg_color(Color::LightRed).unwrap();
        terminal.reset_bg_color().unwrap();
        terminal.set_bg_color(Color::Black).unwrap();
        terminal.set_bg_color(Color::LightGreen).unwrap();

        #[cfg(unix)]
        assert_eq!(
            "\x1B[48;5;9m\x1B[49m\x1B[48;5;0m\x1B[48;5;10m",
            std::str::from_utf8(&terminal.get_buffer_content()).unwrap()
        );
    }
}
