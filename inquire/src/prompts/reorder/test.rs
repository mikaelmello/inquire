//! Test module for the reorderable list prompt functionality.
//!
//! This module contains tests that verify the correct behavior of the `Reorder` prompt,
//! including cursor movement, item reordering, and proper output generation.

use crate::{
    test::fake_backend,
    ui::{Key, KeyModifiers},
    Reorder,
};

#[test]
/// Tests that when no reordering actions are performed, the original order is preserved.
///
/// This test verifies that:
/// - The prompt can be completed without any item movements
/// - The output maintains the original order of items
/// - Basic prompt functionality works correctly
fn no_moves() {
    // Setup a fake backend that simulates a space key press followed by Enter
    // The space key doesn't trigger any reordering action, so items stay in original order
    let mut backend = fake_backend(vec![Key::Char(' ', KeyModifiers::NONE), Key::Enter]);

    let options = vec!["Hello! 111", "Hello! 222"];

    let ans = Reorder::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    // Verify that the order remains unchanged
    assert_eq!(vec!["Hello! 111", "Hello! 222"], ans);
}

#[test]
/// Tests moving the second item to the first position using keyboard controls.
///
/// This test verifies that:
/// - Cursor movement works correctly (Down arrow moves cursor to second item)
/// - Item reordering works correctly (Ctrl+Up moves the selected item up)
/// - The final output reflects the reordered items
/// - Keyboard shortcuts for reordering are properly mapped
fn move_2_to_1() {
    // Setup a fake backend that simulates:
    // 1. Down arrow: move cursor to second item ("Hello! 222")
    // 2. Ctrl+Up: move the currently selected item up one position
    // 3. Enter: submit the reordered list
    let mut backend = fake_backend(vec![
        Key::Down(KeyModifiers::NONE),  // Move cursor to second item
        Key::Up(KeyModifiers::CONTROL), // Move selected item up
        Key::Enter,                     // Submit
    ]);

    let options = vec!["Hello! 111", "Hello! 222"];

    let ans = Reorder::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    // Verify that "Hello! 222" is now first and "Hello! 111" is second
    assert_eq!(vec!["Hello! 222", "Hello! 111"], ans);
}
