use super::Text;
use crate::ui::{Key, KeyModifiers};
use crate::validator::{ErrorMessage, Validation};

fn default<'a, 'b>() -> Text<'a, 'b> {
    Text::new("Question?")
}

macro_rules! text_to_events {
    ($text:expr) => {{
        $text
            .chars()
            .map(|c| Key::Char(c, KeyModifiers::NONE))
            .collect::<Vec<Key>>()
    }};
}

macro_rules! text_test {
    ($name:ident,$input:expr,$output:expr) => {
        text_test! {$name, $input, $output, default()}
    };

    ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
        #[test]
        fn $name() {
            let mut backend = crate::prompts::test::fake_backend($input);

            let ans = $prompt.prompt_with_backend(&mut backend).unwrap();

            assert_eq!($output, ans);
        }
    };
}

text_test!(empty, vec![Key::Enter], "");

text_test!(
    single_letter,
    vec![Key::Char('b', KeyModifiers::NONE), Key::Enter],
    "b"
);

text_test!(
    letters_and_enter,
    text_to_events!("normal input\n"),
    "normal input"
);

text_test!(
    letters_and_enter_with_emoji,
    text_to_events!("with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞\n"),
    "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞"
);

text_test!(
    alt_enter_inserts_newline,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("ab"));
        events.push(Key::Char('\n', KeyModifiers::ALT));
        events.append(&mut text_to_events!("cd"));
        events.push(Key::Enter);
        events
    },
    "ab\ncd"
);

text_test!(
    multiline_input_with_multiple_newlines,
    {
        let mut events = vec![];
        events.append(&mut text_to_events!("line1"));
        events.push(Key::Char('\n', KeyModifiers::ALT));
        events.append(&mut text_to_events!("line2"));
        events.push(Key::Char('\n', KeyModifiers::ALT));
        events.append(&mut text_to_events!("line3"));
        events.push(Key::Enter);
        events
    },
    "line1\nline2\nline3"
);

text_test!(
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
    "normal input"
);

text_test!(
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
    "normal input"
);

text_test!(
    input_correction_after_validation,
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
    Text::new("").with_validator(|ans: &str| match ans.len() {
        len if len > 5 && len < 10 => Ok(Validation::Valid),
        _ => Ok(Validation::Invalid(ErrorMessage::Default)),
    })
);

#[test]
fn tab_before_cursor_keeps_cursor_on_rendered_char() {
    use crate::terminal::{
        test::{emulate_screen, MockTerminal},
        TerminalSize,
    };
    use crate::ui::{Backend, RenderConfig};
    use std::collections::VecDeque;

    let size = TerminalSize::new(80, 24).unwrap();
    let mut output = VecDeque::new();
    let ans;
    {
        let mut keys = text_to_events!("a\tbc");
        keys.push(Key::Left(KeyModifiers::NONE));
        keys.push(Key::Left(KeyModifiers::NONE));
        keys.push(Key::Enter);

        let terminal = MockTerminal::new(&mut output).with_size(size);
        let mut backend =
            Backend::new(keys.into_iter(), terminal, RenderConfig::default()).unwrap();
        ans = default().prompt_with_backend(&mut backend).unwrap();
    }

    assert_eq!("a\tbc", ans);

    let screens = emulate_screen(&output, size);
    // "? Question? a" ends at column 13, the tab advances to column 16,
    // where "b" is rendered. After two Left presses the cursor is on "b".
    let before_enter = &screens[screens.len() - 3];
    assert_eq!("? Question? a   bc", before_enter.rows[0]);
    assert_eq!((0, 16), before_enter.cursor);
}
