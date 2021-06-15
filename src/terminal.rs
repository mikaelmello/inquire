use std::io::{self, stdout, Write};

use termion::{
    color,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl Terminal {
    /// # Errors
    ///
    /// Will return `std::io::Error` if it fails to get terminal size
    pub fn new() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: stdout().into_raw_mode()?,
        })
    }

    #[must_use]
    pub fn size(&self) -> Size {
        self.size
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(&self, x: u16, y: u16) {
        let x = x.saturating_add(1) as u16;
        let y = y.saturating_add(1) as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }

    /// # Errors
    ///
    /// Will return error when call to stdout().flush() fails
    pub fn flush(&self) -> Result<(), std::io::Error> {
        io::stdout().flush()
    }

    /// # Errors
    ///
    /// Will never return Error for now
    pub fn read_key(&self) -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }

    pub fn cursor_hide(&self) {
        print!("{}", termion::cursor::Hide);
    }

    pub fn cursor_show(&self) {
        print!("{}", termion::cursor::Show);
    }

    pub fn clear_current_line(&self) {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn set_bg_color(&self, color: color::Rgb) {
        print!("{}", color::Bg(color));
    }

    pub fn reset_bg_color(&self) {
        print!("{}", color::Bg(color::Reset));
    }

    pub fn set_fg_color(&self, color: color::Rgb) {
        print!("{}", color::Fg(color));
    }

    pub fn reset_fg_color(&self) {
        print!("{}", color::Fg(color::Reset));
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.cursor_show()
    }
}
