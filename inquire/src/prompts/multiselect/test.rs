use crate::{
    formatter::MultiOptionFormatter,
    list_option::ListOption,
    terminal::crossterm::CrosstermTerminal,
    ui::{Backend, RenderConfig},
    MultiSelect,
};
use crossterm::event::{KeyCode, KeyEvent};

#[test]
/// Tests that a closure that actually closes on a variable can be used
/// as a Select formatter.
fn closure_formatter() {
    let read: Vec<KeyEvent> = vec![KeyCode::Char(' '), KeyCode::Enter]
        .into_iter()
        .map(KeyEvent::from)
        .collect();
    let mut read = read.iter();

    let formatted = String::from("Thanks!");
    let formatter: MultiOptionFormatter<'_, i32> = &|_| formatted.clone();

    let options = vec![1, 2, 3];

    let mut write: Vec<u8> = Vec::new();
    let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
    let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

    let ans = MultiSelect::new("Question", options)
        .with_formatter(formatter)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec![ListOption::new(0, 1)], ans);
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

    let ans = MultiSelect::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(Vec::<ListOption<i32>>::new(), ans);
}

#[test]
fn selecting_all_by_default_behavior() {
    let read: Vec<KeyEvent> = [KeyCode::Enter, KeyCode::Enter]
        .iter()
        .map(|c| KeyEvent::from(*c))
        .collect();

    let mut read = read.iter();

    let options = vec![1, 2, 3];

    let mut write: Vec<u8> = Vec::new();
    let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
    let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

    let answer_with_all_selected_by_default = MultiSelect::new("Question", options.clone())
        .with_all_selected_by_default()
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected_result = vec![
        ListOption::new(0, 1),
        ListOption::new(1, 2),
        ListOption::new(2, 3),
    ];

    assert_eq!(expected_result, answer_with_all_selected_by_default);

    let answer_with_none_selected_by_default = MultiSelect::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected_result = Vec::<ListOption<i32>>::new();

    assert_eq!(expected_result, answer_with_none_selected_by_default);
}

#[test]
// Anti-regression test: https://github.com/mikaelmello/inquire/issues/31
fn list_option_indexes_are_relative_to_input_vec() {
    let read: Vec<KeyEvent> = vec![
        KeyCode::Down,
        KeyCode::Char(' '),
        KeyCode::Down,
        KeyCode::Char(' '),
        KeyCode::Enter,
    ]
    .into_iter()
    .map(KeyEvent::from)
    .collect();
    let mut read = read.iter();

    let options = vec![1, 2, 3];

    let mut write: Vec<u8> = Vec::new();
    let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
    let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

    let ans = MultiSelect::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec![ListOption::new(1, 2), ListOption::new(2, 3)], ans);
}
