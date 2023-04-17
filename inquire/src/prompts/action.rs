//! Definitions for the broad Action type which encompasses
//! the directives for prompts.

use std::fmt::Debug;

use crate::ui::Key;

/// Top-level type to describe the directives a prompt
/// receives.
///
/// Each prompt should implement its own custom InnerAction type
/// which is parsed and stored in the Inner variant, if applicable,
/// on the normal execution flow of a prompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action<I>
where
    I: Copy + Clone + PartialEq + Eq,
{
    /// Submits the current prompt answer, finishing the prompt if valid.
    Submit,
    /// Cancels the prompt execution with a graceful shutdown.
    Cancel,
    /// Interrupts the prompt execution without a graceful shutdown.
    Interrupt,
    /// Specialized actions according to the prompt type.
    Inner(I),
}

impl<I> Action<I>
where
    I: Copy + Clone + PartialEq + Eq,
{
    /// Derives a prompt action from a Key event.
    pub fn from_key<C>(key: Key, config: &C) -> Option<Action<I>>
    where
        I: InnerAction<C>,
    {
        match key {
            Key::Cancel => Some(Action::Cancel),
            Key::Interrupt => Some(Action::Interrupt),
            Key::Submit => Some(Action::Submit),
            key => I::from_key(key, config).map(Action::Inner),
        }
    }
}

/// InnerActions are specialized prompt actions.
///
/// They must provide an implementation to optionally derive an action
/// from a key event.
pub trait InnerAction<C>
where
    Self: Sized + Copy + Clone + PartialEq + Eq,
{
    /// Derives a prompt action from a Key event and the prompt configuration.
    fn from_key(key: Key, config: &C) -> Option<Self>
    where
        Self: Sized;
}
