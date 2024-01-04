use std::{fmt::Display, io::Result};

use crate::{
    error::InquireResult,
    ui::{dimension::Dimension, InputReader, Styled},
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

pub type TerminalSize = Dimension;

pub trait Terminal: Sized {
    fn get_size(&self) -> Result<TerminalSize>;

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
