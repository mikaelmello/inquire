use crate::{
    ui::{Key, KeyModifiers},
    InputAction,
};

pub trait ParseKey: Sized {
    fn from_key(key: Key) -> Option<Self>;
}

pub enum Action<InnerAction> {
    Control(ControlAction),
    Input(InputAction),
    Inner(InnerAction),
}

/// Top-level type to describe the directives a prompt
/// receives.
///
/// Each prompt should implement its own custom InnerAction type
/// which is parsed and stored in the Inner variant, if applicable,
/// on the normal execution flow of a prompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ControlAction {
    /// Submits the current prompt answer, finishing the prompt if valid.
    Submit,
    /// Cancels the prompt execution with a graceful shutdown.
    Cancel,
    /// Interrupts the prompt execution without a graceful shutdown.
    Interrupt,
}

impl ParseKey for ControlAction {
    fn from_key(key: Key) -> Option<Self> {
        match key {
            Key::Enter | Key::Char('j', KeyModifiers::CONTROL) => Some(Self::Submit),
            Key::Escape | Key::Char('d', KeyModifiers::CONTROL) => Some(Self::Cancel),
            Key::Char('c', KeyModifiers::CONTROL) => Some(Self::Interrupt),
            _ => None,
        }
    }
}
