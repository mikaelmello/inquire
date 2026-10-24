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

password_test!(
    alt_enter_inserts_newline,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("ab"));
        events.push(Key::Char('\n', KeyModifiers::ALT));
        events.append(&mut text_to_events!("cd"));
        events.push(Key::Enter);
        events
    },
    "ab\ncd",
    Password::new("").without_confirmation()
);

password_test!(
    multiline_password_with_confirmation,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("ab"));
        events.push(Key::Char('\n', KeyModifiers::ALT));
        events.append(&mut text_to_events!("cd"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("ab"));
        events.push(Key::Char('\n', KeyModifiers::ALT));
        events.append(&mut text_to_events!("cd"));
        events.push(Key::Enter);
        events
    },
    "ab\ncd",
    Password::new("")
);

password_test!(
    #[should_panic(expected = "EOF")]
    multiline_password_with_mismatched_confirmation,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("ab"));
        events.push(Key::Char('\n', KeyModifiers::ALT));
        events.append(&mut text_to_events!("cd"));
        events.push(Key::Enter);
        events.append(&mut text_to_events!("ab"));
        events.push(Key::Char('\n', KeyModifiers::ALT));
        events.append(&mut text_to_events!("XX"));
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

/// Replays the tokens written to a [`MockTerminal`] and returns the cursor
/// position, relative to where the prompt started, each time the cursor is
/// shown, i.e. at the end of each rendered frame.
fn cursor_positions_when_shown(
    output: &std::collections::VecDeque<crate::terminal::test::MockTerminalToken>,
) -> Vec<(u16, u16)> {
    use crate::terminal::test::MockTerminalToken;
    use unicode_width::UnicodeWidthChar;

    let (mut row, mut col) = (0u16, 0u16);
    let mut positions = vec![];

    for token in output {
        match token {
            MockTerminalToken::Text(styled) => {
                for c in styled.content.chars() {
                    match c {
                        '\r' => col = 0,
                        '\n' => row += 1,
                        c => col += c.width().unwrap_or(0) as u16,
                    }
                }
            }
            MockTerminalToken::CursorUp(n) => row -= n,
            MockTerminalToken::CursorDown(n) => row += n,
            MockTerminalToken::CursorLeft(n) => col -= n,
            MockTerminalToken::CursorRight(n) => col += n,
            MockTerminalToken::CursorMoveToColumn(n) => col = *n,
            MockTerminalToken::CursorShow => positions.push((row, col)),
            MockTerminalToken::ClearLine
            | MockTerminalToken::ClearUntilNewLine
            | MockTerminalToken::CursorHide => {}
        }
    }

    positions
}

/// Runs `prompt` against a mock terminal and returns the cursor position at
/// the end of every frame rendered while the prompt was still active.
fn active_cursor_positions(prompt: Password<'_>, input: Vec<Key>) -> Vec<(u16, u16)> {
    use crate::terminal::test::MockTerminal;
    use crate::ui::{Backend, RenderConfig};

    // One frame is rendered before the first key, then one after each key
    // except the final `Enter`, which renders the submitted answer instead.
    let active_frames = input.len();

    let mut output = std::collections::VecDeque::new();
    {
        let terminal = MockTerminal::new(&mut output);
        let mut backend =
            Backend::new(input.into_iter(), terminal, RenderConfig::default()).unwrap();
        prompt.prompt_with_backend(&mut backend).unwrap();
    }

    let mut positions = cursor_positions_when_shown(&output);
    assert!(positions.len() > active_frames);
    positions.truncate(active_frames);
    positions
}

// Anti-regression tests for https://github.com/mikaelmello/inquire/issues/348:
// in hidden mode the cursor was left at the start of the line instead of
// after the prompt text.
const PROMPT: &str = "Encryption Key:";
// "? " prefix, the prompt text, and the space after it.
const CURSOR_AFTER_PROMPT: (u16, u16) = (0, 2 + PROMPT.len() as u16 + 1);

#[test]
fn hidden_cursor_is_after_prompt() {
    let positions = active_cursor_positions(
        Password::new(PROMPT).without_confirmation(),
        vec![Key::Enter],
    );

    assert_eq!(vec![CURSOR_AFTER_PROMPT], positions);
}

#[test]
fn hidden_cursor_is_after_prompt_with_help_message() {
    let positions = active_cursor_positions(
        Password::new(PROMPT)
            .with_display_mode(crate::PasswordDisplayMode::Hidden)
            .with_help_message("It is used to encrypt your files")
            .without_confirmation(),
        vec![Key::Enter],
    );

    assert_eq!(vec![CURSOR_AFTER_PROMPT], positions);
}

#[test]
fn hidden_cursor_stays_after_prompt_while_typing() {
    let mut input: Vec<Key> = text_to_events!("secret");
    input.push(Key::Enter);

    let positions = active_cursor_positions(
        Password::new(PROMPT)
            .with_display_mode(crate::PasswordDisplayMode::Hidden)
            .with_help_message("It is used to encrypt your files")
            .without_confirmation(),
        input,
    );

    // One frame before any input, then one per typed character.
    assert_eq!(vec![CURSOR_AFTER_PROMPT; 7], positions);
}
