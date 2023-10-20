//! Definitions for the broad Action type which encompasses
//! the directives for prompts.

use std::fmt::Debug;

use crate::ui::{Key, KeyModifiers};

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
        I: InnerAction<Config = C>,
    {
        match key {
            Key::Enter | Key::Char('j', KeyModifiers::CONTROL) => Some(Action::Submit),
            Key::Escape | Key::Char('g', KeyModifiers::CONTROL) => Some(Action::Cancel),
            Key::Char('c', KeyModifiers::CONTROL) => Some(Action::Interrupt),
            key => I::from_key(key, config).map(Action::Inner),
        }
    }
}

/// InnerActions are specialized prompt actions.
///
/// They must provide an implementation to optionally derive an action
/// from a key event.
pub trait InnerAction
where
    Self: Sized + Copy + Clone + PartialEq + Eq,
{
    /// Configuration type for the prompt.
    ///
    /// This is used to derive the action from a key event.
    type Config;

    /// Derives a prompt action from a Key event and the prompt configuration.
    fn from_key(key: Key, config: &Self::Config) -> Option<Self>
    where
        Self: Sized;
}
