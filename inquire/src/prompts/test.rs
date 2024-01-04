use crate::{
    terminal::crossterm::CrosstermTerminal,
    ui::{Backend, InputReader, Key, RenderConfig},
};

impl<T> InputReader for T
where
    T: Iterator<Item = Key>,
{
    fn read_key(&mut self) -> crate::error::InquireResult<Key> {
        let key = self.next();

        match key {
            Some(key) => Ok(key),
            None => panic!("EOF"),
        }
    }
}

pub fn fake_backend(input: Vec<Key>) -> Backend<'static, impl InputReader, CrosstermTerminal> {
    let output = CrosstermTerminal::new_in_memory_output();
    Backend::new(input.into_iter(), output, RenderConfig::default()).unwrap()
}
