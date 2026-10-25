use crate::{
    formatter::MultiOptionFormatter,
    list_option::ListOption,
    terminal::test::match_text,
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

#[test]
// Anti-regression test: https://github.com/mikaelmello/inquire/issues/294
fn first_option_renders_on_new_line_without_filtering() {
    use crate::{
        terminal::test::MockTerminal,
        ui::{Backend, InputReader, Key, RenderConfig},
    };
    use std::collections::VecDeque;

    // Create input reader
    struct MockInputReader;
    impl InputReader for MockInputReader {
        fn read_key(&mut self) -> crate::error::InquireResult<Key> {
            Ok(Key::Enter)
        }
    }

    let mut output = VecDeque::new();
    let terminal = MockTerminal::new(&mut output);
    let input_reader = MockInputReader;
    let render_config = RenderConfig::default();

    let options = vec!["Option 1", "Option 2", "Option 3"];

    {
        let mut backend = Backend::new(input_reader, terminal, render_config).unwrap();
        let _ans = MultiSelect::new("Select rating", options)
            .without_filtering()
            .prompt_with_backend(&mut backend)
            .unwrap();
    }

    // The key assertion is that there should be a newline after the prompt
    // Before the bug fix, the first option would appear on the same line as the prompt
    // After the bug fix, there should be a newline separating them

    // We're testing that with without_filtering(), the newline appears after the prompt
    match_text(&mut output, "?");
    match_text(&mut output, " ");
    match_text(&mut output, "Select rating");
    match_text(&mut output, " ");
    match_text(&mut output, "\r");
    match_text(&mut output, "\n");
}

mod tabs {
    use std::collections::VecDeque;

    use crate::{
        list_option::ListOption,
        terminal::{
            test::{emulate_screen, MockTerminal, ScreenSnapshot},
            TerminalSize,
        },
        ui::{Backend, Key, KeyModifiers, RenderConfig},
        MultiSelect,
    };

    const WIDTH: u16 = 40;

    fn run(options: Vec<&str>, keys: Vec<Key>) -> (Vec<ListOption<&str>>, Vec<ScreenSnapshot>) {
        let size = TerminalSize::new(WIDTH, 40).unwrap();
        let mut output = VecDeque::new();
        let ans;
        {
            let terminal = MockTerminal::new(&mut output).with_size(size);
            let mut backend =
                Backend::new(keys.into_iter(), terminal, RenderConfig::default()).unwrap();
            ans = MultiSelect::new("Pick users", options)
                .without_filtering()
                .prompt_with_backend(&mut backend)
                .unwrap();
        }
        (ans, emulate_screen(&output, size))
    }

    fn down_up_cycles() -> Vec<Key> {
        let mut keys = vec![];
        for _ in 0..3 {
            keys.push(Key::Down(KeyModifiers::NONE));
            keys.push(Key::Up(KeyModifiers::NONE));
        }
        keys.push(Key::Enter);
        keys
    }

    #[test]
    // Anti-regression test: https://github.com/mikaelmello/inquire/issues/344
    fn redraws_of_options_with_tabs_match_space_only_control() {
        // long enough to wrap in a 40-column terminal
        let tabbed = vec![
            "alice\tadmin\tmaintains the release pipeline",
            "bob\tviewer\treads the dashboards",
            "charlotte\tops\ton call for the database cluster",
        ];
        // the same options, with tabs expanded after the 6-column "> [ ] " prefix
        let spaced = vec![
            "alice     admin   maintains the release pipeline",
            "bob       viewer  reads the dashboards",
            "charlotte ops     on call for the database cluster",
        ];

        let (_, tabbed_screens) = run(tabbed, down_up_cycles());
        let (_, spaced_screens) = run(spaced, down_up_cycles());

        assert_eq!(spaced_screens, tabbed_screens);
        // after each down/up cycle, the screen is the same
        assert_eq!(tabbed_screens[2], tabbed_screens[4]);
        assert_eq!(tabbed_screens[2], tabbed_screens[6]);
    }

    #[test]
    fn tabs_align_option_columns() {
        let (_, screens) = run(
            vec!["alice\tadmin", "bob\tviewer", "charlotte\tops"],
            vec![Key::Enter],
        );

        let first_frame = &screens[0];
        let columns: Vec<Option<usize>> = ["admin", "viewer", "ops"]
            .iter()
            .map(|field| {
                first_frame
                    .rows
                    .iter()
                    .find_map(|row| row.find(&format!(" {}", field)))
                    .map(|idx| idx + 1)
            })
            .collect();

        assert_eq!(vec![Some(16), Some(16), Some(16)], columns);
    }

    #[test]
    fn tabs_are_kept_in_answer() {
        let (ans, _) = run(
            vec!["alice\tadmin", "bob\tviewer"],
            vec![Key::Char(' ', KeyModifiers::NONE), Key::Enter],
        );

        assert_eq!(vec![ListOption::new(0, "alice\tadmin")], ans);
    }
}

#[test]
fn ansi_escape_codes_do_not_match_filter() {
    let option = "\x1b[32mApple\x1b[0m";

    assert_eq!(
        None,
        MultiSelect::<&str>::DEFAULT_SCORER("32m", &option, option, 0)
    );
}

#[test]
fn ansi_escape_codes_do_not_affect_matching_visible_text() {
    let colored = "\x1b[32mApple\x1b[0m";
    let plain = "Apple";
    let score = |input, option| MultiSelect::<&str>::DEFAULT_SCORER(input, &option, option, 0);

    assert!(score("app", colored).is_some());
    assert_eq!(score("app", plain), score("app", colored));
    assert_eq!(score("Apple", plain), score("Apple", colored));
}
