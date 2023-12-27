use std::convert::TryFrom;

use crossterm::event::KeyEvent;

use crate::{
    terminal::crossterm::CrosstermTerminal,
    ui::{Backend, Key, RenderConfig},
};

pub fn fake_backend(input: Vec<Key>) -> Backend<'static, CrosstermTerminal> {
    let events: Vec<KeyEvent> = input
        .into_iter()
        .map(|k| KeyEvent::try_from(k).expect("Could not convert Key to KeyEvent"))
        .collect();
    let terminal = CrosstermTerminal::new_with_io(events.into());

    Backend::new(terminal, RenderConfig::default()).unwrap()
}
