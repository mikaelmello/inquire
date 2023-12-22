use crate::{
    ui::{Key, KeyModifiers},
    InnerAction, InputAction,
};

pub trait ParseKey: Sized {
    fn from_key(key: Key) -> Option<Self>;
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

pub enum Action<InnerAction> {
    Control(ControlAction),
    Input(InputAction),
    Inner(InnerAction),
}

impl<InnerAction> ParseKey for Action<InnerAction>
where
    InnerAction: ParseKey,
{
    fn from_key(key: Key) -> Option<Self> {
        ControlAction::from_key(key)
            .map(Self::Control)
            .or_else(|| InnerAction::from_key(key).map(Self::Inner))
            .or_else(|| InputAction::from_key(key, &()).map(Self::Input))
    }
}
