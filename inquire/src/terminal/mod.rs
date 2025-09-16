use std::{fmt::Display, io::Result};

use crate::{
    error::InquireResult,
    ui::{InputReader, Styled},
};

#[cfg(feature = "crossterm")]
#[cfg_attr(docsrs, doc(cfg(feature = "crossterm")))]
pub mod crossterm;

#[cfg(feature = "termion")]
#[cfg_attr(docsrs, doc(cfg(feature = "termion")))]
pub mod termion;

#[cfg(feature = "console")]
#[cfg_attr(docsrs, doc(cfg(feature = "console")))]
pub mod console;

#[cfg(test)]
pub(crate) mod test;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    width: u16,
    height: u16,
}

impl TerminalSize {
    /**
     * Returns None if the width or height is 0
     */
    pub fn new(width: u16, height: u16) -> Option<Self> {
        if width == 0 || height == 0 {
            None
        } else {
            Some(Self { width, height })
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
        }
    }
}

pub trait Terminal: Sized {
    fn get_size(&self) -> Result<Option<TerminalSize>>;

    fn write<T: Display>(&mut self, val: T) -> Result<()>;
    fn write_styled<T: Display>(&mut self, val: &Styled<T>) -> Result<()>;

    fn clear_line(&mut self) -> Result<()>;
    fn clear_until_new_line(&mut self) -> Result<()>;

    fn cursor_hide(&mut self) -> Result<()>;
    fn cursor_show(&mut self) -> Result<()>;
    fn cursor_up(&mut self, cnt: u16) -> Result<()>;
    fn cursor_down(&mut self, cnt: u16) -> Result<()>;
    fn cursor_left(&mut self, cnt: u16) -> Result<()>;
    fn cursor_right(&mut self, cnt: u16) -> Result<()>;
    #[allow(unused)]
    fn cursor_move_to_column(&mut self, idx: u16) -> Result<()>;

    fn flush(&mut self) -> Result<()>;
}

pub fn get_default_terminal() -> InquireResult<(impl InputReader, impl Terminal)> {
    #[cfg(feature = "crossterm")]
    return Ok((
        crossterm::CrosstermKeyReader::new(),
        crossterm::CrosstermTerminal::new()?,
    ));

    #[cfg(all(feature = "termion", not(feature = "crossterm")))]
    return Ok((
        termion::TermionKeyReader::new()?,
        termion::TermionTerminal::new()?,
    ));

    #[cfg(all(
        feature = "console",
        not(feature = "termion"),
        not(feature = "crossterm")
    ))]
    {
        let console_terminal = console::ConsoleTerminal::new();
        let console_key_reader = console_terminal.clone();
        return Ok((console_key_reader, console_terminal));
    }

    #[cfg(all(
        not(feature = "crossterm"),
        not(feature = "termion"),
        not(feature = "console")
    ))]
    {
        compile_error!("At least one of crossterm, termion or console must be enabled");

        // this is here to silence an additional compilation error
        // when no terminals are enabled. it complains about mismatched
        // return types.
        Err(crate::error::InquireError::InvalidConfiguration(
            "Missing terminal backend".into(),
        ))
    }
}
