#[cfg(feature = "crossterm")]
use crate::{
    date_utils::get_current_date,
    terminal::crossterm::CrosstermTerminal,
    ui::{Backend, RenderConfig},
    validator::Validation,
    DateSelect,
};
use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent};

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
            let read: Vec<KeyEvent> = $input.into_iter().map(KeyEvent::from).collect();
            let mut read = read.iter();

            let mut write: Vec<u8> = Vec::new();
            let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
            let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

            let ans = $prompt.prompt_with_backend(&mut backend).unwrap();

            assert_eq!($output, ans);
        }
    };
}

date_test!(today_date, vec![KeyCode::Enter], get_current_date());

date_test!(
    custom_default_date,
    vec![KeyCode::Enter],
    NaiveDate::from_ymd(2021, 1, 9),
    DateSelect::new("Date").with_default(NaiveDate::from_ymd(2021, 1, 9))
);

#[test]
/// Tests that a closure that actually closes on a variable can be used
/// as a DateSelect validator.
fn closure_validator() {
    let read: Vec<KeyEvent> = vec![KeyCode::Enter, KeyCode::Left, KeyCode::Enter]
        .into_iter()
        .map(KeyEvent::from)
        .collect();
    let mut read = read.iter();

    let today_date = get_current_date();

    let validator = move |d| {
        if today_date > d {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Date must be in the past".into()))
        }
    };

    let mut write: Vec<u8> = Vec::new();
    let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
    let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

    let ans = DateSelect::new("Question")
        .with_validator(validator)
        .prompt_with_backend(&mut backend)
        .unwrap();

    assert_eq!(today_date.pred(), ans);
}
