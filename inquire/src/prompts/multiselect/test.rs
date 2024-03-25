use crate::{
    formatter::MultiOptionFormatter,
    list_option::ListOption,
    test::fake_backend,
    ui::{Key, KeyModifiers},
    MultiSelect,
};

#[test]
/// Tests that a closure that actually closes on a variable can be used
/// as a Select formatter.
fn closure_formatter() {
    let mut backend = fake_backend(vec![Key::Char(' ', KeyModifiers::NONE), Key::Enter]);

    let formatted = String::from("Thanks!");
    let formatter: MultiOptionFormatter<'_, i32> = &|_| formatted.clone();

    let options = vec![1, 2, 3];

    let ans = MultiSelect::new("Question", options)
        .with_formatter(formatter)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec![ListOption::new(0, 1)], ans);
}

#[test]
// Anti-regression test: https://github.com/mikaelmello/inquire/issues/30
fn down_arrow_on_empty_list_does_not_panic() {
    let mut backend = fake_backend(vec![
        Key::Char('9', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Backspace,
        Key::Char('3', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Backspace,
        Key::Enter,
    ]);

    let options = vec![1, 2, 3];

    let ans = MultiSelect::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(Vec::<ListOption<i32>>::new(), ans);
}

#[test]
fn selecting_all_by_default_behavior() {
    let mut backend = fake_backend(vec![Key::Enter, Key::Enter]);
    let options = vec![1, 2, 3];

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
    let mut backend = fake_backend(vec![
        Key::Down(KeyModifiers::NONE),
        Key::Char(' ', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Char(' ', KeyModifiers::NONE),
        Key::Enter,
    ]);

    let options = vec![1, 2, 3];

    let ans = MultiSelect::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec![ListOption::new(1, 2), ListOption::new(2, 3)], ans);
}

#[test]
// Anti-regression test: https://github.com/mikaelmello/inquire/issues/195
fn starting_cursor_is_respected() {
    let mut backend = fake_backend(vec![Key::Char(' ', KeyModifiers::NONE), Key::Enter]);
    let options = vec![1, 2, 3];

    let ans = MultiSelect::new("Question", options)
        .with_starting_cursor(2)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec![ListOption::new(2, 3)], ans);
}

#[test]
fn naive_assert_fuzzy_match_as_default_scorer() {
    let mut backend = fake_backend(vec![
        Key::Char('w', KeyModifiers::NONE),
        Key::Char('r', KeyModifiers::NONE),
        Key::Char('r', KeyModifiers::NONE),
        Key::Char('y', KeyModifiers::NONE),
        Key::Char(' ', KeyModifiers::NONE),
        Key::Enter,
    ]);

    let options = vec![
        "Banana",
        "Apple",
        "Strawberry",
        "Grapes",
        "Lemon",
        "Tangerine",
        "Watermelon",
        "Orange",
        "Pear",
        "Avocado",
        "Pineapple",
    ];

    let ans = MultiSelect::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec![ListOption::new(2, "Strawberry")], ans);
}

#[test]
fn chars_do_not_affect_prompt_without_filtering() {
    let mut backend = fake_backend(vec![
        Key::Char('w', KeyModifiers::NONE),
        Key::Char('r', KeyModifiers::NONE),
        Key::Char('r', KeyModifiers::NONE),
        Key::Char('y', KeyModifiers::NONE),
        Key::Char(' ', KeyModifiers::NONE),
        Key::Enter,
    ]);

    let options = vec![
        "Banana",
        "Apple",
        "Strawberry",
        "Grapes",
        "Lemon",
        "Tangerine",
        "Watermelon",
        "Orange",
        "Pear",
        "Avocado",
        "Pineapple",
    ];

    let ans = MultiSelect::new("Question", options)
        .without_filtering()
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(vec![ListOption::new(0, "Banana")], ans);
}

#[test]
fn keep_filter_false_behavior() {
    let mut backend = fake_backend(vec![
        Key::Char('1', KeyModifiers::NONE), // filter to option 1
        Key::Char(' ', KeyModifiers::NONE), // toggle, filter input is reset to empty
        Key::Char('2', KeyModifiers::NONE), // filter is now '2' and shows option 2
        Key::Char(' ', KeyModifiers::NONE), // toggle
        Key::Enter,
    ]);

    let options = vec![1, 2, 3, 4, 5];

    let ans = MultiSelect::new("Question", options)
        .with_keep_filter(false)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected_answer = vec![ListOption::new(0, 1), ListOption::new(1, 2)];
    assert_eq!(expected_answer, ans);
}

#[test]
fn keep_filter_true_behavior() {
    let mut backend = fake_backend(vec![
        Key::Char('1', KeyModifiers::NONE), // filter to option 1
        Key::Char(' ', KeyModifiers::NONE), // toggle, filter input is NOT reset
        Key::Char('2', KeyModifiers::NONE), // filter is now '12' and shows no option
        Key::Char(' ', KeyModifiers::NONE), // should be no-op
        Key::Enter,
    ]);

    let options = vec![1, 2, 3, 4, 5];

    let ans = MultiSelect::new("Question", options)
        .with_keep_filter(true)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected_answer = vec![ListOption::new(0, 1)];
    assert_eq!(expected_answer, ans);
}

#[test]
fn keep_filter_should_be_true_by_default() {
    let mut backend = fake_backend(vec![
        Key::Char('1', KeyModifiers::NONE), // filter to option 1
        Key::Char(' ', KeyModifiers::NONE), // toggle, filter input is NOT reset
        Key::Char('2', KeyModifiers::NONE), // filter is now '12' and shows no option
        Key::Char(' ', KeyModifiers::NONE), // should be no-op
        Key::Enter,
    ]);

    let options = vec![1, 2, 3, 4, 5];

    let prompt = MultiSelect::new("Question", options);
    assert!(prompt.keep_filter);
    let ans = prompt.prompt_with_backend(&mut backend).unwrap();

    let expected_answer = vec![ListOption::new(0, 1)];
    assert_eq!(expected_answer, ans);
}

#[test]
// Anti-regression test: https://github.com/mikaelmello/inquire/issues/238
fn keep_filter_false_should_reset_option_list() {
    let mut backend = fake_backend(vec![
        Key::Char('3', KeyModifiers::NONE), // filter to option 3
        Key::Char(' ', KeyModifiers::NONE), // toggle option 3, filter input is reset
        Key::Char(' ', KeyModifiers::NONE), // toggle option 1 after option list is reset
        Key::Enter,
    ]);

    let options = vec![1, 2, 3, 4, 5];

    let ans = MultiSelect::new("Question", options)
        .with_keep_filter(false)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected_answer = vec![ListOption::new(0, 1), ListOption::new(2, 3)];
    assert_eq!(expected_answer, ans);
}
