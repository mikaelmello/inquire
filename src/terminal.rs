use std::{
    fmt::Display,
    io::{self, stdout, Write},
};

use termion::{
    color::{self, Color},
    cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

use crate::terminal;

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
    applied_fgs: Vec<String>,
    applied_bgs: Vec<String>,
    applied_styles: Vec<Style>,
}

#[derive(Copy, Clone)]
pub enum Style {
    Bold,
    Italic,
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
            applied_bgs: vec![],
            applied_fgs: vec![],
            applied_styles: vec![],
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

    pub fn cursor_up(&self) {
        print!("{}", cursor::Up(1));
    }

    pub fn cursor_horizontal_reset(&self) {
        print!("\r");
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

    pub fn set_style(&mut self, style: Style) {
        self.applied_styles.push(style);
        let style: Box<dyn Display> = match style {
            Style::Bold => Box::new(termion::style::Bold),
            Style::Italic => Box::new(termion::style::Italic),
        };

        print!("{}", style);
    }

    pub fn undo_style(&mut self) {
        if let Some(st) = self.applied_styles.pop() {
            match st {
                Style::Bold => print!("{}", termion::style::NoBold),
                Style::Italic => print!("{}", termion::style::NoItalic),
            }
        }
    }

    pub fn reset_style(&mut self) {
        self.applied_styles.clear();
        print!("{}", termion::style::Reset);
    }

    pub fn set_bg_color<C: Color>(&mut self, color: C) {
        let fmt = format!("{}", color::Bg(color));
        print!("{}", fmt);
        self.applied_bgs.push(fmt);
    }

    pub fn undo_bg_color(&mut self) {
        self.applied_bgs.pop();

        match self.applied_bgs.last() {
            Some(bg) => print!("{}", bg),
            None => print!("{}", color::Bg(color::Reset)),
        }
    }

    pub fn reset_bg_color(&mut self) {
        self.applied_bgs.clear();
        print!("{}", color::Bg(color::Reset));
    }

    pub fn set_fg_color<C: Color>(&mut self, color: C) {
        let fmt = format!("{}", color::Fg(color));
        print!("{}", fmt);
        self.applied_fgs.push(fmt);
    }

    pub fn undo_fg_color(&mut self) {
        self.applied_fgs.pop();

        match self.applied_fgs.last() {
            Some(fg) => print!("{}", fg),
            None => print!("{}", color::Fg(color::Reset)),
        }
    }

    pub fn reset_fg_color(&mut self) {
        self.applied_fgs.clear();
        print!("{}", color::Fg(color::Reset));
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.cursor_show()
    }
}
