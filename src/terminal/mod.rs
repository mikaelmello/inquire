use std::{fmt::Display, io::Result};

use crate::{
    error::InquireResult,
    ui::{Attributes, Color, Key, Styled},
};

use self::crossterm::CrosstermTerminal;

const INITIAL_IN_MEMORY_CAPACITY: usize = 2048;

pub mod crossterm;

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
    fn write_styled<'s, T: Display>(&mut self, val: &'s Styled<T>) -> Result<()>;

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

pub fn get_default_terminal() -> InquireResult<impl Terminal> {
    CrosstermTerminal::new()
}
