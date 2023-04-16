use crate::{
    formatter::OptionFormatter,
    list_option::ListOption,
    terminal::crossterm::CrosstermTerminal,
    ui::{Backend, RenderConfig},
    Select,
};
use crossterm::event::{KeyCode, KeyEvent};

#[test]
/// Tests that a closure that actually closes on a variable can be used
/// as a Select formatter.
fn closure_formatter() {
    let read: Vec<KeyEvent> = vec![KeyCode::Down, KeyCode::Enter]
        .into_iter()
        .map(KeyEvent::from)
        .collect();
    let mut read = read.iter();

    let formatted = String::from("Thanks!");
    let formatter: OptionFormatter<i32> = &|_| formatted.clone();

    let options = vec![1, 2, 3];

    let mut write: Vec<u8> = Vec::new();
    let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
    let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

    let ans = Select::new("Question", options)
        .with_formatter(formatter)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(ListOption::new(1, 2), ans);
}

#[test]
// Anti-regression test: https://github.com/mikaelmello/inquire/issues/29
fn enter_arrow_on_empty_list_does_not_panic() {
    let read: Vec<KeyEvent> = [
        KeyCode::Char('9'),
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::Char('3'),
        KeyCode::Enter,
    ]
    .iter()
    .map(|c| KeyEvent::from(*c))
    .collect();

    let mut read = read.iter();

    let options = vec![1, 2, 3];

    let mut write: Vec<u8> = Vec::new();
    let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
    let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

    let ans = Select::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(ListOption::new(2, 3), ans);
}

#[test]
// Anti-regression test: https://github.com/mikaelmello/inquire/issues/30
fn down_arrow_on_empty_list_does_not_panic() {
    let read: Vec<KeyEvent> = [
        KeyCode::Char('9'),
        KeyCode::Down,
        KeyCode::Backspace,
        KeyCode::Char('3'),
        KeyCode::Down,
        KeyCode::Backspace,
        KeyCode::Enter,
    ]
    .iter()
    .map(|c| KeyEvent::from(*c))
    .collect();

    let mut read = read.iter();

    let options = vec![1, 2, 3];

    let mut write: Vec<u8> = Vec::new();
    let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
    let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

    let ans = Select::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(ListOption::new(0, 1), ans);
}
