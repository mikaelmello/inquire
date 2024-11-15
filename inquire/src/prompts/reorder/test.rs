use crate::{
    test::fake_backend,
    ui::{Key, KeyModifiers},
    ReorderableList,
};

#[test]
/// Tests that a closure that actually closes on a variable can be used
/// as a Select formatter.
fn no_moves() {
    let mut backend = fake_backend(vec![Key::Char(' ', KeyModifiers::NONE), Key::Enter]);

    let options = vec!["Hello! 111".into(), "Hello! 222".into()];

    let ans = ReorderableList::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec!["Hello! 111".to_string(), "Hello! 222".to_string()], ans);
}

#[test]
/// Tests that a closure that actually closes on a variable can be used
/// as a Select formatter.
fn move_2_to_1() {
    let mut backend = fake_backend(vec![Key::Down(KeyModifiers::NONE), Key::Up(KeyModifiers::CONTROL), Key::Enter]);

    let options = vec!["Hello! 111".into(), "Hello! 222".into()];

    let ans = ReorderableList::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec!["Hello! 222".to_string(), "Hello! 111".to_string()], ans);
}
