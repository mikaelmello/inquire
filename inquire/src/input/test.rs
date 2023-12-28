use unicode_segmentation::UnicodeSegmentation;

use super::Input;
use crate::{
    input::{InputActionResult, LineDirection, Magnitude},
    InputAction,
};

#[test]
fn move_previous_word() {
    let content = "great 🌍, 🍞, 🚗, 1231321📞, 🎉, 🍆xsa232 s2da ake iak eaik";

    let assert = |expected, initial| {
        let mut input = Input::new_with(content).with_cursor(initial);

        let result = input.handle(InputAction::MoveCursor(
            Magnitude::Word,
            LineDirection::Left,
        ));
        let cursor_moved = result == InputActionResult::ContentChanged
            || result == InputActionResult::PositionChanged;

        assert_eq!(expected != initial, cursor_moved,
            "cursor_moved '{}' is not equal to expected '{}' because of initial and expected cursors '{}' and '{}'",
            cursor_moved, expected != initial, initial, expected);
        assert_eq!(
            expected,
            input.cursor(),
            "unexpected result cursor from initial {initial}",
        );
    };

    for i in 0..16 {
        assert(0, i);
    }
    for i in 16..30 {
        assert(15, i);
    }
    for i in 30..37 {
        assert(29, i);
    }
    for i in 37..42 {
        assert(36, i);
    }
    for i in 42..46 {
        assert(41, i);
    }
    for i in 46..50 {
        assert(45, i);
    }
    for i in 50..54 {
        assert(49, i);
    }
}

#[test]
// https://github.com/mikaelmello/inquire/issues/5
fn regression_issue_5() {
    let heart = '♥';
    let vs16 = '\u{fe0f}';

    let heart_without_vs16 = String::from(heart);
    let heart_with_vs16 = "♥️";
    // let heart_with_vs16_char = '♥️'; // this doesn't compile because there are 2 chars
    let built_heart_with_vs16 = {
        let mut s = String::from(heart);
        s.push(vs16);
        s
    };

    assert_eq!(6, heart_with_vs16.len());
    assert_eq!(2, heart_with_vs16.chars().count());
    assert_eq!(1, heart_with_vs16.graphemes(true).count());
    assert_eq!(&built_heart_with_vs16, heart_with_vs16);

    let mut input = Input::new();
    assert_eq!(0, input.length);
    assert_eq!(0, input.cursor);
    assert_eq!("", input.content);

    input.handle(InputAction::Write(heart));
    {
        assert_eq!(1, input.length);
        assert_eq!(1, input.cursor);
        assert_eq!(heart_without_vs16, input.content);
        assert_ne!(heart_with_vs16, input.content);
        assert!(input.content.find(vs16).is_none());
    }

    input.handle(InputAction::Write(vs16));
    {
        assert_eq!(1, input.length);
        assert_eq!(1, input.cursor);
        assert_ne!(heart_without_vs16, input.content);
        assert_eq!(heart_with_vs16, input.content);
        assert!(input.content.find(vs16).is_some());
    }
}

#[test]
fn new_is_empty() {
    let input = Input::new();
    assert_eq!(0, input.length());
    assert_eq!(0, input.cursor());
    assert_eq!("", input.content());
    assert_eq!("", input.pre_cursor());
    assert_eq!(None, input.placeholder());
    assert!(input.is_empty());
}

#[test]
fn new_with_content_is_correctly_initialized() {
    let content = "great idea!";
    let input = Input::new_with(content);
    assert_eq!(11, input.length());
    assert_eq!(11, input.cursor());
    assert_eq!(content, input.content());
}

#[test]
fn with_cursor_panics_if_out_of_bounds() {
    let input = Input::new_with("great idea!");
    assert_eq!(11, input.length());
    assert_eq!(11, input.cursor());
    assert_eq!("great idea!", input.content());
    assert_eq!("great idea!", input.pre_cursor());
    assert_eq!(None, input.placeholder());

    assert!(std::panic::catch_unwind(|| input.with_cursor(12)).is_err());
}

#[test]
fn with_cursor_is_correctly_initialized() {
    let input = Input::new_with("great idea!").with_cursor(7);
    assert_eq!(11, input.length());
    assert_eq!(7, input.cursor());
    assert_eq!("great idea!", input.content());
    assert_eq!("great i", input.pre_cursor());
}

#[test]
fn with_placeholder_is_correctly_initialized() {
    let input = Input::new_with("great idea!").with_placeholder("placeholder");
    assert_eq!(11, input.length());
    assert_eq!(11, input.cursor());
    assert_eq!("great idea!", input.content());
    assert_eq!("placeholder", input.placeholder().unwrap());
}

#[test]
fn clear_makes_content_empty() {
    let mut input = Input::new_with("great idea!").with_cursor(7);
    assert_eq!(11, input.length());
    assert_eq!(7, input.cursor());
    assert_eq!("great idea!", input.content());
    assert_eq!("great i", input.pre_cursor());
    assert_eq!(None, input.placeholder());

    input.clear();

    assert_eq!(0, input.length());
    assert_eq!(0, input.cursor());
    assert_eq!("", input.content());
    assert_eq!("", input.pre_cursor());
    assert_eq!(None, input.placeholder());
}

#[test]
fn clear_does_not_affect_placeholder() {
    let mut input = Input::new_with("great idea!")
        .with_cursor(7)
        .with_placeholder("placeholder");
    assert_eq!(11, input.length());
    assert_eq!(7, input.cursor());
    assert_eq!("great idea!", input.content());
    assert_eq!("great i", input.pre_cursor());
    assert_eq!(Some("placeholder"), input.placeholder());

    input.clear();

    assert_eq!(0, input.length());
    assert_eq!(0, input.cursor());
    assert_eq!("", input.content());
    assert_eq!("", input.pre_cursor());
    assert_eq!(Some("placeholder"), input.placeholder());
}

#[test]
fn move_cursor_action_tests() {
    let mut input = Input::new_with("great idea! you are a genius")
        .with_cursor(15)
        .with_placeholder("placeholder");

    assert_eq!("great idea! you are a genius", input.content());
    assert_eq!("great idea! you", input.pre_cursor());
    assert_eq!(15, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Char,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea! yo", input.pre_cursor());
    assert_eq!(14, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Char,
        LineDirection::Right,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea! you", input.pre_cursor());
    assert_eq!(15, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Word,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea! ", input.pre_cursor());
    assert_eq!(12, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Word,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great ", input.pre_cursor());
    assert_eq!(6, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Word,
        LineDirection::Right,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea", input.pre_cursor());
    assert_eq!(10, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Word,
        LineDirection::Right,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea! you", input.pre_cursor());
    assert_eq!(15, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Line,
        LineDirection::Right,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea! you are a genius", input.pre_cursor());
    assert_eq!(28, input.cursor());

    // these should not move the cursor
    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Line,
        LineDirection::Right,
    ));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("great idea! you are a genius", input.pre_cursor());
    assert_eq!(28, input.cursor());
    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Word,
        LineDirection::Right,
    ));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("great idea! you are a genius", input.pre_cursor());
    assert_eq!(28, input.cursor());
    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Char,
        LineDirection::Right,
    ));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("great idea! you are a genius", input.pre_cursor());
    assert_eq!(28, input.cursor());

    // going to beginning of line
    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Line,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("", input.pre_cursor());
    assert_eq!(0, input.cursor());

    // these should not move the cursor
    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Line,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("", input.pre_cursor());
    assert_eq!(0, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Word,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("", input.pre_cursor());
    assert_eq!(0, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Char,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("", input.pre_cursor());
    assert_eq!(0, input.cursor());

    assert_eq!("great idea! you are a genius", input.content());
}

#[test]
fn delete_action_tests() {
    let mut input = Input::new_with("great idea! you are a genius")
        .with_cursor(15)
        .with_placeholder("placeholder");

    assert_eq!("great idea! you are a genius", input.content());
    assert_eq!("great idea! you", input.pre_cursor());
    assert_eq!(15, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Char, LineDirection::Left));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea! yo are a genius", input.content());
    assert_eq!("great idea! yo", input.pre_cursor());
    assert_eq!(14, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Char, LineDirection::Right));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea! yoare a genius", input.content());
    assert_eq!("great idea! yo", input.pre_cursor());
    assert_eq!(14, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Word, LineDirection::Left));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea! are a genius", input.content());
    assert_eq!("great idea! ", input.pre_cursor());
    assert_eq!(12, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Word, LineDirection::Right));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea!  a genius", input.content());
    assert_eq!("great idea! ", input.pre_cursor());
    assert_eq!(12, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Word, LineDirection::Right));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea!  genius", input.content());
    assert_eq!("great idea! ", input.pre_cursor());
    assert_eq!(12, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Line, LineDirection::Right));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea! ", input.content());
    assert_eq!("great idea! ", input.pre_cursor());
    assert_eq!(12, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Line, LineDirection::Right));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("great idea! ", input.content());
    assert_eq!("great idea! ", input.pre_cursor());
    assert_eq!(12, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Word, LineDirection::Right));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("great idea! ", input.content());
    assert_eq!("great idea! ", input.pre_cursor());
    assert_eq!(12, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Char, LineDirection::Right));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("great idea! ", input.content());
    assert_eq!("great idea! ", input.pre_cursor());
    assert_eq!(12, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Line, LineDirection::Left));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("", input.content());
    assert_eq!("", input.pre_cursor());
    assert_eq!(0, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Line, LineDirection::Left));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("", input.content());
    assert_eq!("", input.pre_cursor());
    assert_eq!(0, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Word, LineDirection::Left));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("", input.content());
    assert_eq!("", input.pre_cursor());
    assert_eq!(0, input.cursor());

    let result = input.handle(InputAction::Delete(Magnitude::Char, LineDirection::Left));
    assert_eq!(InputActionResult::Clean, result);
    assert_eq!("", input.content());
    assert_eq!("", input.pre_cursor());
    assert_eq!(0, input.cursor());
}

#[test]
fn generic_user_scenario() {
    let mut input = Input::new_with("great idea! you are a genius")
        .with_cursor(15)
        .with_placeholder("placeholder");

    assert_eq!("great idea! you are a genius", input.content());
    assert_eq!("great idea! you", input.pre_cursor());
    assert_eq!(15, input.cursor());

    let result = input.handle(InputAction::Write('a'));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea! youa are a genius", input.content());
    assert_eq!("great idea! youa", input.pre_cursor());
    assert_eq!(16, input.cursor());

    let result = input.handle(InputAction::Write('b'));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea! youab are a genius", input.content());
    assert_eq!("great idea! youab", input.pre_cursor());
    assert_eq!(17, input.cursor());

    let result = input.handle(InputAction::Write('c'));
    assert_eq!(InputActionResult::ContentChanged, result);
    assert_eq!("great idea! youabc are a genius", input.content());
    assert_eq!("great idea! youabc", input.pre_cursor());
    assert_eq!(18, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Char,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea! youabc are a genius", input.content());
    assert_eq!("great idea! youab", input.pre_cursor());
    assert_eq!(17, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Char,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea! youabc are a genius", input.content());
    assert_eq!("great idea! youa", input.pre_cursor());
    assert_eq!(16, input.cursor());

    let result = input.handle(InputAction::MoveCursor(
        Magnitude::Char,
        LineDirection::Left,
    ));
    assert_eq!(InputActionResult::PositionChanged, result);
    assert_eq!("great idea! youabc are a genius", input.content());
    assert_eq!("great idea! you", input.pre_cursor());
    assert_eq!(15, input.cursor());
}
