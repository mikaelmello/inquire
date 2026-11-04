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

/// Terminal size in rows and columns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    width: u16,
    height: u16,
}

impl TerminalSize {
    /// Returns None if the width or height is 0
    pub fn new(width: u16, height: u16) -> Option<Self> {
        if width == 0 || height == 0 {
            None
        } else {
            Some(Self { width, height })
        }
    }

    /// Get number of columns available
    pub fn width(&self) -> u16 {
        self.width
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

/// Trait for terminal provider implementations
pub trait Terminal: Sized {
    /// Try to get the terminal size
    fn get_size(&self) -> Result<Option<TerminalSize>>;

    /// Write given Display to the terminal
    fn write<T: Display>(&mut self, val: T) -> Result<()>;
    /// Write given Styled to the terminal
    fn write_styled<T: Display>(&mut self, val: &Styled<T>) -> Result<()>;

    /// Clear the current line
    fn clear_line(&mut self) -> Result<()>;
    /// Clear the from cursor position to line end
    fn clear_until_new_line(&mut self) -> Result<()>;

    /// Hide the cursor
    fn cursor_hide(&mut self) -> Result<()>;
    /// Show the cursor
    fn cursor_show(&mut self) -> Result<()>;
    /// Move the cursor up by cnt
    fn cursor_up(&mut self, cnt: u16) -> Result<()>;
    /// Move the cursor down by cnt
    fn cursor_down(&mut self, cnt: u16) -> Result<()>;
    /// Move the cursor left by cnt
    fn cursor_left(&mut self, cnt: u16) -> Result<()>;
    /// Move the cursor right by cnt
    fn cursor_right(&mut self, cnt: u16) -> Result<()>;
    /// Move the cursor to position idxc
    fn cursor_move_to_column(&mut self, idx: u16) -> Result<()>;

    /// Flush pending changes
    fn flush(&mut self) -> Result<()>;
}

/// Get a new instance of configured terminal provider
///
/// # Example:
/// ```rust no_run
/// let (_, mut terminal) = inquire::get_default_terminal()?;
/// terminal.write_styled(
///     &Styled::new("Hii helloo!").with_fg(Colors::LightYellow),
/// )?;
/// ```
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
        std::unreachable!();
    }
}
