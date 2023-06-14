use bitflags::bitflags;
use dyn_clone::DynClone;

use crate::{error::GenericError, CustomUserError};

// Using the same struct, but without importing, to cut prompts' direct dependencies to crossterm
// https://github.com/crossterm-rs/crossterm/blob/e1260446e94e9a8f7809fef61dc1369b6f8d6e12/src/event.rs#L376-L385
bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const NONE = 0b0000_0000;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Key {
    Escape,
    Enter,
    Backspace,
    Tab,
    Delete(KeyModifiers),
    Home,
    End,
    PageUp,
    PageDown,
    Up(KeyModifiers),
    Down(KeyModifiers),
    Left(KeyModifiers),
    Right(KeyModifiers),
    Char(char, KeyModifiers),
    #[deprecated(note = "If the key you want isn't mapped, please open a PR.")]
    Any,
}

pub trait KeyMapper<Action>: DynClone {
    fn map(&self, key: Key) -> Result<Option<Action>, GenericError>;
}

impl<Action> Clone for Box<dyn KeyMapper<Action>> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl<Action, F> KeyMapper<Action> for F
where
    F: Fn(Key) -> Result<Option<Action>, CustomUserError> + Clone,
{
    fn map(&self, key: Key) -> Result<Option<Action>, CustomUserError> {
        (self)(key)
    }
}
