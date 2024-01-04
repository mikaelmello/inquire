use super::Password;
use crate::ui::{Key, KeyModifiers};
use crate::validator::{ErrorMessage, Validation};

macro_rules! text_to_events {
    ($text:expr) => {{
        $text
            .chars()
            .map(|c| Key::Char(c, KeyModifiers::NONE))
            .collect()
    }};
}

macro_rules! password_test {
    ($(#[$meta:meta])? $name:ident,$input:expr,$output:expr,$prompt:expr) => {
        #[test]
        $(#[$meta])?
        fn $name() {
            let mut backend = crate::prompts::test::fake_backend($input);

            let ans = $prompt.prompt_with_backend(&mut backend).unwrap();

            assert_eq!($output, ans);
        }
    };
}

password_test!(
    empty,
    vec![Key::Enter],
    "",
    Password::new("").without_confirmation()
);

password_test!(
    single_letter,
    vec![Key::Char('b', KeyModifiers::NONE), Key::Enter],
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
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.append(&mut text_to_events!("normal input"));
        events.push(Key::Enter);
        events
    },
    "normal input",
    Password::new("").without_confirmation()
);

password_test!(
    input_and_excessive_correction,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.append(&mut text_to_events!("normal input"));
        events.push(Key::Enter);
        events
    },
    "normal input",
    Password::new("").without_confirmation()
);

password_test!(
    input_correction_after_validation_when_masked,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890"));
        events.push(Key::Enter);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.append(&mut text_to_events!("yes"));
        events.push(Key::Enter);
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
        events.append(&mut text_to_events!("1234567890"));
        events.push(Key::Enter);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.push(Key::Backspace);
        events.append(&mut text_to_events!("yes"));
        events.push(Key::Enter);
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
        events.append(&mut text_to_events!("1234567890"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("yesyes"));
        events.push(Key::Enter);
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
        events.append(&mut text_to_events!("1234567890"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("1234567890"));
        events.push(Key::Enter);
        events
    },
    "1234567890",
    Password::new("")
);

password_test!(
    #[should_panic(expected = "EOF")]
    input_confirmation_different,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("1234567890"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("abcdefghij"));
        events.push(Key::Enter);
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
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("anor2"));
        events.push(Key::Enter);
        // The problem is that the 1st input values were not cleared
        // and the lack of a change in the 1st prompt can be confusing.
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
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
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("anor2"));
        events.push(Key::Enter);
        // The problem is that the 1st input values were not cleared
        // and the lack of a change in the 1st prompt can be confusing.
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
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
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("anor2"));
        events.push(Key::Enter);
        // The problem is that the 1st input values were not cleared
        // and the lack of a change in the 1st prompt can be confusing.
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("anor"));
        events.push(Key::Enter);
        events
    },
    "anor",
    Password::new("").with_display_mode(crate::PasswordDisplayMode::Masked)
);
