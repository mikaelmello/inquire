//! Definitions for the broad Action type which encompasses
//! the directives for prompts.

use std::fmt::Debug;

use crate::{
    ui::{Key, KeyMapper, KeyModifiers},
    CustomUserError,
};

/// Top-level type to describe the directives a prompt
/// receives.
///
/// Each prompt should implement its own custom InnerAction type
/// which is parsed and stored in the Inner variant, if applicable,
/// on the normal execution flow of a prompt.
#[derive(Debug)]
pub enum Action<I> {
    /// Submits the current prompt answer, finishing the prompt if valid.
    Submit,
    /// Cancels the prompt execution with a graceful shutdown.
    Cancel,
    /// Interrupts the prompt execution without a graceful shutdown.
    Interrupt,
    /// Specialized actions according to the prompt type.
    Inner(I),
    /// External action, which can be hooked into the prompt execution
    /// to provide additional functionality.
    External,
}

pub(crate) type PromptKeyMapper<I> = Box<dyn KeyMapper<Action<I>>>;

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

#[derive(Clone)]
pub(crate) struct DefaultKeyHandler<C>
where
    C: Clone,
{
    config: C,
}

impl<C> DefaultKeyHandler<C>
where
    C: Clone,
{
    pub fn new(config: C) -> Self {
        Self { config }
    }
}

impl<I, C> KeyMapper<Action<I>> for DefaultKeyHandler<C>
where
    I: InnerAction<C>,
    C: Clone,
{
    fn map(&self, key: Key) -> Result<Option<Action<I>>, CustomUserError> {
        let action = match key {
            Key::Enter => Some(Action::Submit),
            Key::Escape => Some(Action::Cancel),
            Key::Char('c', KeyModifiers::CONTROL) => Some(Action::Interrupt),
            key => I::from_key(key, &self.config).map(Action::Inner),
        };

        Ok(action)
    }
}
