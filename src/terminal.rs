use std::{
    fmt::Display,
    io::{stdin, stdout, BufRead, Error, Read, Write},
};

use termion::{
    color::{self, Color},
    cursor,
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
};

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    #[allow(unused)]
    size: Size,
    reader: Box<dyn Read>,
    writer: Box<dyn Write>,
    applied_fgs: Vec<String>,
    applied_bgs: Vec<String>,
    applied_styles: Vec<Style>,
}

#[derive(Copy, Clone)]
pub enum Style {
    Bold,
    #[allow(unused)]
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
            reader: Box::new(stdin()),
            writer: Box::new(stdout().into_raw_mode()?),
            applied_bgs: vec![],
            applied_fgs: vec![],
            applied_styles: vec![],
        })
    }

    #[allow(unused)]
    pub fn with_writer<W: 'static + Write>(mut self, writer: W) -> Self {
        self.writer = Box::new(writer);
        self
    }

    #[allow(unused)]
    pub fn with_reader<W: 'static + BufRead>(mut self, reader: W) -> Self {
        self.reader = Box::new(reader);
        self
    }

    #[must_use]
    #[allow(unused)]
    pub fn size(&self) -> Size {
        self.size
    }

    pub fn cursor_up(&mut self) -> Result<(), Error> {
        write!(self.writer, "{}", cursor::Up(1))
    }

    pub fn cursor_horizontal_reset(&mut self) -> Result<(), Error> {
        write!(self.writer, "\x1b[0G")
    }

    /// # Errors
    ///
    /// Will return error when call to stdout().flush() fails
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.writer.flush()
    }

    /// # Errors
    ///
    /// Will never return Error for now
    pub fn read_key(&mut self) -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = (*self.reader).keys().next() {
                return key;
            }
        }
    }

    pub fn write(&mut self, val: &str) -> Result<(), std::io::Error> {
        write!(self.writer, "{}", val)
    }

    pub fn cursor_hide(&mut self) -> Result<(), std::io::Error> {
        write!(self.writer, "{}", termion::cursor::Hide)
    }

    pub fn cursor_show(&mut self) -> Result<(), std::io::Error> {
        write!(self.writer, "{}", termion::cursor::Show)
    }

    pub fn clear_current_line(&mut self) -> Result<(), std::io::Error> {
        write!(self.writer, "{}", termion::clear::CurrentLine)
    }

    pub fn set_style(&mut self, style: Style) -> Result<(), std::io::Error> {
        self.applied_styles.push(style);
        let style: Box<dyn Display> = match style {
            Style::Bold => Box::new(termion::style::Bold),
            Style::Italic => Box::new(termion::style::Italic),
        };

        write!(self.writer, "{}", style)
    }

    pub fn undo_style(&mut self) -> Result<(), std::io::Error> {
        match self.applied_styles.pop() {
            Some(st) => match st {
                Style::Bold => write!(self.writer, "{}", termion::style::NoBold),
                Style::Italic => write!(self.writer, "{}", termion::style::NoItalic),
            },
            None => Ok(()),
        }
    }

    #[allow(unused)]
    pub fn reset_style(&mut self) -> Result<(), std::io::Error> {
        self.applied_styles.clear();
        write!(self.writer, "{}", termion::style::Reset)
    }

    pub fn set_bg_color<C: Color>(&mut self, color: C) -> Result<(), std::io::Error> {
        let fmt = format!("{}", color::Bg(color));
        let res = write!(self.writer, "{}", fmt);
        self.applied_bgs.push(fmt);

        res
    }

    pub fn undo_bg_color(&mut self) -> Result<(), std::io::Error> {
        self.applied_bgs.pop();

        match self.applied_bgs.last() {
            Some(bg) => write!(self.writer, "{}", bg),
            None => write!(self.writer, "{}", color::Bg(color::Reset)),
        }
    }

    #[allow(unused)]
    pub fn reset_bg_color(&mut self) -> Result<(), std::io::Error> {
        self.applied_bgs.clear();
        write!(self.writer, "{}", color::Bg(color::Reset))
    }

    pub fn set_fg_color<C: Color>(&mut self, color: C) -> Result<(), std::io::Error> {
        let fmt = format!("{}", color::Fg(color));
        let res = write!(self.writer, "{}", fmt);
        self.applied_fgs.push(fmt);

        res
    }

    pub fn undo_fg_color(&mut self) -> Result<(), std::io::Error> {
        self.applied_fgs.pop();

        match self.applied_fgs.last() {
            Some(fg) => write!(self.writer, "{}", fg),
            None => write!(self.writer, "{}", color::Fg(color::Reset)),
        }
    }

    #[allow(unused)]
    pub fn reset_fg_color(&mut self) -> Result<(), std::io::Error> {
        self.applied_fgs.clear();
        write!(self.writer, "{}", color::Fg(color::Reset))
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.cursor_show();
    }
}
