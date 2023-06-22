//! Definitions for the broad Action type which encompasses
//! the directives for prompts.

use std::{fmt::Debug, marker::PhantomData};

use crate::ui::{Key, KeyModifiers};

/// Top-level type to describe the directives a prompt
/// receives.
///
/// Each prompt should implement its own custom InnerAction type
/// which is parsed and stored in the Inner variant, if applicable,
/// on the normal execution flow of a prompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action<I> {
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
            Key::Enter => Some(Action::Submit),
            Key::Escape => Some(Action::Cancel),
            Key::Char('c', KeyModifiers::CONTROL) => Some(Action::Interrupt),
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

pub(crate) trait ActionMapper<A> {
    fn get_action(&self, key: Key) -> Option<A>;
}

pub(crate) struct BuiltinActionMapper<IA, M>
where
    M: ActionMapper<IA>,
{
    __phantom: PhantomData<IA>,
    inner_action_mapper: M,
}

impl<IA, M> BuiltinActionMapper<IA, M>
where
    M: ActionMapper<IA>,
{
    pub fn new(inner_action_mapper: M) -> Self {
        Self {
            __phantom: PhantomData,
            inner_action_mapper,
        }
    }
}

impl<IA, M> ActionMapper<Action<IA>> for BuiltinActionMapper<IA, M>
where
    M: ActionMapper<IA>,
{
    fn get_action(&self, key: Key) -> Option<Action<IA>> {
        match key {
            Key::Enter => Some(Action::Submit),
            Key::Escape => Some(Action::Cancel),
            Key::Char('c', KeyModifiers::CONTROL) => Some(Action::Interrupt),
            key => self.inner_action_mapper.get_action(key).map(Action::Inner),
        }
    }
}
