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

    input.insert(heart);
    {
        assert_eq!(1, input.length);
        assert_eq!(1, input.cursor);
        assert_eq!(heart_without_vs16, input.content);
        assert_ne!(heart_with_vs16, input.content);
        assert!(input.content.find(vs16).is_none());
    }

    input.insert(vs16);
    {
        assert_eq!(1, input.length);
        assert_eq!(1, input.cursor);
        assert_ne!(heart_without_vs16, input.content);
        assert_eq!(heart_with_vs16, input.content);
        assert!(input.content.find(vs16).is_some());
    }
}
