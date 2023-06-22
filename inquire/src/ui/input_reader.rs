use std::marker::PhantomData;

use crate::{error::InquireResult, terminal::Terminal, ActionMapper, InquireError};

use super::Key;

pub trait InputReader<A> {
    fn next_action(&mut self) -> InquireResult<Option<A>>;
}

pub(crate) struct TerminalInputReader<T, A, M>
where
    T: Terminal,
    M: ActionMapper<A>,
{
    __phantom: PhantomData<A>,
    terminal: T,
    action_mapper: M,
}

pub(crate) struct MockInputReader<A> {
    actions: Vec<Option<A>>,
}

impl<A> MockInputReader<A> {
    pub fn new(actions: Vec<Option<A>>) -> Self {
        Self { actions }
    }

    pub fn new_from_keys(
        keys: &mut dyn Iterator<Item = &Key>,
        action_mapper: &dyn ActionMapper<A>,
    ) -> Self {
        let actions = keys.map(|k| action_mapper.get_action(*k)).collect();

        Self { actions }
    }
}

impl<A> InputReader<A> for MockInputReader<A> {
    fn next_action(&mut self) -> InquireResult<Option<A>> {
        self.actions.pop().ok_or_else(|| {
            InquireError::IO(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "No more actions",
            ))
        })
    }
}
