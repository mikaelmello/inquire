use crate::{
    date_utils::get_current_date,
    test::fake_backend,
    ui::{Key, KeyModifiers},
    validator::Validation,
    DateSelect,
};
use chrono::NaiveDate;

fn default<'a>() -> DateSelect<'a> {
    DateSelect::new("Question?")
}

macro_rules! date_test {
    ($name:ident,$input:expr,$output:expr) => {
        date_test! {$name, $input, $output, default()}
    };

    ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
        #[test]
        fn $name() {
            let mut backend = fake_backend($input);

            let ans = $prompt.prompt_with_backend(&mut backend).unwrap();

            assert_eq!($output, ans);
        }
    };
}

date_test!(today_date, vec![Key::Enter], get_current_date());

date_test!(
    custom_default_date,
    vec![Key::Enter],
    NaiveDate::from_ymd_opt(2021, 1, 9).unwrap(),
    DateSelect::new("Date").with_default(NaiveDate::from_ymd_opt(2021, 1, 9).unwrap())
);

#[test]
/// Tests that a closure that actually closes on a variable can be used
/// as a DateSelect validator.
fn closure_validator() {
    let mut backend = fake_backend(vec![Key::Enter, Key::Left(KeyModifiers::NONE), Key::Enter]);

    let today_date = get_current_date();

    let validator = move |d| {
        if today_date > d {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Date must be in the past".into()))
        }
    };

    let ans = DateSelect::new("Question")
        .with_validator(validator)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(today_date.pred_opt().unwrap(), ans);
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn daily_navigation_checks() {
    let input = vec![
        Key::Left(KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = fake_backend(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(NaiveDate::from_ymd_opt(2023, 1, 20).unwrap(), ans);
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn weekly_navigation_checks() {
    let input = vec![
        Key::Up(KeyModifiers::NONE),
        Key::Up(KeyModifiers::NONE),
        Key::Up(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = fake_backend(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(NaiveDate::from_ymd_opt(2023, 2, 19).unwrap(), ans);
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn monthly_navigation_checks() {
    let input = vec![
        Key::Char('[', KeyModifiers::NONE),
        Key::Char(']', KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Char(']', KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = fake_backend(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(NaiveDate::from_ymd_opt(2022, 11, 15).unwrap(), ans);
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn yearly_navigation_checks() {
    let input = vec![
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = fake_backend(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), ans);
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn naive_navigation_combination() {
    let input = vec![
        // start: 2023-01-15
        Key::Up(KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Up(KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Char(']', KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char(']', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Up(KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = fake_backend(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(), ans);
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn emacs_naive_navigation_combination() {
    let input = vec![
        // start: 2023-01-15
        Key::Char('p', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::ALT),
        Key::Char('p', KeyModifiers::CONTROL),
        Key::Char('b', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::ALT),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('b', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::ALT),
        Key::Char('b', KeyModifiers::CONTROL),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::ALT),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::CONTROL),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('p', KeyModifiers::CONTROL),
        Key::Enter,
    ];
    let mut backend = fake_backend(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(), ans);
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn vim_naive_navigation_combination() {
    let input = vec![
        // start: 2023-01-15
        Key::Char('k', KeyModifiers::NONE),
        Key::Char('b', KeyModifiers::ALT),
        Key::Char('k', KeyModifiers::NONE),
        Key::Char('h', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::ALT),
        Key::Char('b', KeyModifiers::ALT),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('h', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('b', KeyModifiers::ALT),
        Key::Char('h', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('b', KeyModifiers::ALT),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::ALT),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('k', KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = fake_backend(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(), ans);
}
