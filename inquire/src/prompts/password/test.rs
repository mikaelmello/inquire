use super::Password;
use crate::{
    terminal::crossterm::CrosstermTerminal,
    ui::{Backend, RenderConfig},
    validator::{ErrorMessage, Validation},
};
use crossterm::event::{KeyCode, KeyEvent};

macro_rules! text_to_events {
    ($text:expr) => {{
        $text.chars().map(KeyCode::Char)
    }};
}

macro_rules! password_test {
    ($(#[$meta:meta])? $name:ident,$input:expr,$output:expr,$prompt:expr) => {
        #[test]
        $(#[$meta])?
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

password_test!(
    empty,
    vec![KeyCode::Enter],
    "",
    Password::new("").without_confirmation()
);

password_test!(
    single_letter,
    vec![KeyCode::Char('b'), KeyCode::Enter],
    "b",
    Password::new("").without_confirmation()
);

password_test!(
    letters_and_enter,
    text_to_events!("normal input\n"),
    "normal input",
    Password::new("").without_confirmation()
);

password_test!(
    letters_and_enter_with_emoji,
    text_to_events!("with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞\n"),
    "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞",
    Password::new("").without_confirmation()
);

password_test!(
    input_and_correction,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.append(&mut text_to_events!("normal input").collect());
        events.push(KeyCode::Enter);
        events
    },
    "normal input",
    Password::new("").without_confirmation()
);

password_test!(
    input_and_excessive_correction,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.append(&mut text_to_events!("normal input").collect());
        events.push(KeyCode::Enter);
        events
    },
    "normal input",
    Password::new("").without_confirmation()
);

password_test!(
    input_correction_after_validation_when_masked,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.append(&mut text_to_events!("yes").collect());
        events.push(KeyCode::Enter);
        events
    },
    "12345yes",
    Password::new("")
        .with_display_mode(crate::PasswordDisplayMode::Masked)
        .without_confirmation()
        .with_validator(|ans: &str| match ans.len() {
            len if len > 5 && len < 10 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid(ErrorMessage::Default)),
        })
);

password_test!(
    input_correction_after_validation_when_full,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.push(KeyCode::Backspace);
        events.append(&mut text_to_events!("yes").collect());
        events.push(KeyCode::Enter);
        events
    },
    "12345yes",
    Password::new("")
        .with_display_mode(crate::PasswordDisplayMode::Full)
        .without_confirmation()
        .with_validator(|ans: &str| match ans.len() {
            len if len > 5 && len < 10 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid(ErrorMessage::Default)),
        })
);

password_test!(
    input_correction_after_validation_when_hidden,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("yesyes").collect());
        events.push(KeyCode::Enter);
        events
    },
    "yesyes",
    Password::new("")
        .with_display_mode(crate::PasswordDisplayMode::Hidden)
        .without_confirmation()
        .with_validator(|ans: &str| match ans.len() {
            len if len > 5 && len < 10 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid(ErrorMessage::Default)),
        })
);

password_test!(
    input_confirmation_same,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events
    },
    "1234567890",
    Password::new("")
);

password_test!(
    #[should_panic(expected = "Custom stream of characters has ended")]
    input_confirmation_different,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("abcdefghij").collect());
        events.push(KeyCode::Enter);
        events
    },
    "",
    Password::new("")
);

// Anti-regression test for UX issue: https://github.com/mikaelmello/inquire/issues/149
password_test!(
    prompt_with_hidden_should_clear_on_mismatch,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("anor2").collect());
        events.push(KeyCode::Enter);
        // The problem is that the 1st input values were not cleared
        // and the lack of a change in the 1st prompt can be confusing.
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events
    },
    "anor",
    Password::new("").with_display_mode(crate::PasswordDisplayMode::Hidden)
);

// Anti-regression test for UX issue: https://github.com/mikaelmello/inquire/issues/149
password_test!(
    prompt_with_full_should_clear_1st_on_mismatch,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("anor2").collect());
        events.push(KeyCode::Enter);
        // The problem is that the 1st input values were not cleared
        // and the lack of a change in the 1st prompt can be confusing.
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events
    },
    "anor",
    Password::new("").with_display_mode(crate::PasswordDisplayMode::Full)
);

// Anti-regression test for UX issue: https://github.com/mikaelmello/inquire/issues/149
password_test!(
    prompt_with_masked_should_clear_1st_on_mismatch,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("anor2").collect());
        events.push(KeyCode::Enter);
        // The problem is that the 1st input values were not cleared
        // and the lack of a change in the 1st prompt can be confusing.
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events.append(&mut text_to_events!("anor").collect());
        events.push(KeyCode::Enter);
        events
    },
    "anor",
    Password::new("").with_display_mode(crate::PasswordDisplayMode::Masked)
);
