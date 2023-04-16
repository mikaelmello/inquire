use super::Key;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action<I>
where
    I: Copy + Clone + PartialEq + Eq,
{
    Submit,
    Cancel,
    Interrupt,
    Inner(I),
}

impl<I> Action<I>
where
    I: Copy + Clone + PartialEq + Eq,
{
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

pub trait InnerAction<C>
where
    Self: Sized + Copy + Clone + PartialEq + Eq,
{
    fn from_key(key: Key, config: &C) -> Option<Self>
    where
        Self: Sized;
}
