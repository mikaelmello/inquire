use crate::{
    error::InquireResult,
    {Action, InnerAction},
};

pub trait InputReader<I> {
    fn next_action<C>(&mut self, config: &C) -> InquireResult<Option<Action<I>>>
    where
        I: InnerAction<Config = C>;
}

impl<I, T> InputReader<I> for T
where
    T: Iterator<Item = Action<I>>,
    I: Copy + Clone + PartialEq + Eq,
{
    fn next_action<C>(&mut self, _config: &C) -> InquireResult<Option<Action<I>>>
    where
        I: InnerAction<Config = C>,
    {
        Ok(self.next())
    }
}
