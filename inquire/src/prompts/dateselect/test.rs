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
/// Tests the behaviour of several keybindings in a admittedly naive way.
fn naive_keybinding_checks() {
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
