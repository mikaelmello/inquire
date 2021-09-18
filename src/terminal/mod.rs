use std::{fmt::Display, io::Result};

use crate::{
    error::InquireResult,
    ui::{Key, Styled},
};

const INITIAL_IN_MEMORY_CAPACITY: usize = 2048;

#[cfg(feature = "crossterm")]
#[cfg_attr(docsrs, doc(cfg(feature = "crossterm")))]
pub mod crossterm;

#[cfg(feature = "termion")]
#[cfg_attr(docsrs, doc(cfg(feature = "termion")))]
pub mod termion;

#[cfg(feature = "console")]
#[cfg_attr(docsrs, doc(cfg(feature = "console")))]
pub mod console;

pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

pub trait Terminal: Sized {
    fn cursor_up(&mut self, cnt: u16) -> Result<()>;
    fn cursor_down(&mut self, cnt: u16) -> Result<()>;
    fn cursor_move_to_column(&mut self, idx: u16) -> Result<()>;
    fn read_key(&mut self) -> Result<Key>;
    fn flush(&mut self) -> Result<()>;

    fn get_size(&self) -> Result<TerminalSize>;

    fn get_in_memory_content(&self) -> &str;
    fn clear_in_memory_content(&mut self);

    fn write<T: Display>(&mut self, val: T) -> Result<()>;
    fn write_styled<T: Display>(&mut self, val: &Styled<T>) -> Result<()>;

    fn clear_current_line(&mut self) -> Result<()>;

    fn cursor_hide(&mut self) -> Result<()>;
    fn cursor_show(&mut self) -> Result<()>;
}

pub fn get_default_terminal() -> InquireResult<impl Terminal> {
    #[cfg(feature = "crossterm")]
    return crossterm::CrosstermTerminal::new();

    #[cfg(all(feature = "termion", not(feature = "crossterm")))]
    return termion::TermionTerminal::new();

    #[cfg(all(
        feature = "console",
        not(feature = "termion"),
        not(feature = "crossterm")
    ))]
    return Ok(console::ConsoleTerminal::new());

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
