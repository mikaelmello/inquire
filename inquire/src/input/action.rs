use crate::{
    ui::{Key, KeyModifiers},
    InnerAction,
};

use super::{LineDirection, Magnitude};

/// Set of actions for a text input handler.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputAction {
    /// Deletes a substring of the input according to the magnitude and the
    /// direction to delete.
    Delete(Magnitude, LineDirection),
    /// Moves the cursor according to the magnitude and the direction to move.
    MoveCursor(Magnitude, LineDirection),
    /// Writes a character to the content, according to the current cursor
    /// position.
    Write(char),
}

impl InnerAction for InputAction {
    type Config = ();

    fn from_key(key: Key, _config: &()) -> Option<Self>
    where
        Self: Sized,
    {
        let action = match key {
            Key::Backspace => Self::Delete(Magnitude::Char, LineDirection::Left),
            Key::Char('h', m) if m.contains(KeyModifiers::CONTROL) => {
                // Ctrl+Backspace is tricky, we don't want to handle a Ctrl+H
                // but also don't want ctrl+h to simply write h.
                // Let's catch this combination and ignore it.
                return None;
            }

            Key::Delete(m) if m.contains(KeyModifiers::CONTROL) => {
                Self::Delete(Magnitude::Word, LineDirection::Right)
            }
            Key::Delete(_) => Self::Delete(Magnitude::Char, LineDirection::Right),

            Key::Home => Self::MoveCursor(Magnitude::Line, LineDirection::Left),
            Key::Left(m) if m.contains(KeyModifiers::CONTROL) => {
                Self::MoveCursor(Magnitude::Word, LineDirection::Left)
            }
            Key::Left(_) => Self::MoveCursor(Magnitude::Char, LineDirection::Left),

            Key::End => Self::MoveCursor(Magnitude::Line, LineDirection::Right),
            Key::Right(m) if m.contains(KeyModifiers::CONTROL) => {
                Self::MoveCursor(Magnitude::Word, LineDirection::Right)
            }
            Key::Right(_) => Self::MoveCursor(Magnitude::Char, LineDirection::Right),

            Key::Char(c, _) => Self::Write(c),
            _ => return None,
        };

        Some(action)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn backspace_results_in_delete_char_left() {
        assert_eq!(
            InputAction::from_key(Key::Backspace, &()),
            Some(InputAction::Delete(Magnitude::Char, LineDirection::Left))
        );
    }

    #[test]
    fn delete_results_in_delete_char_right() {
        assert_eq!(
            InputAction::from_key(Key::Delete(KeyModifiers::NONE), &()),
            Some(InputAction::Delete(Magnitude::Char, LineDirection::Right))
        );
    }

    #[test]
    fn ctrl_delete_results_in_delete_word_right() {
        assert_eq!(
            InputAction::from_key(Key::Delete(KeyModifiers::CONTROL), &()),
            Some(InputAction::Delete(Magnitude::Word, LineDirection::Right))
        );
    }

    #[test]
    fn ctrl_backspace_does_nothing() {
        // Ctrl+Backspace is tricky, we don't want to handle a Ctrl+H
        // but also don't want ctrl+h to simply write h.
        // Let's catch this combination and ignore it.
        assert_eq!(
            InputAction::from_key(Key::Char('h', KeyModifiers::CONTROL), &()),
            None
        );
    }

    #[test]
    fn horizontal_arrows_results_in_move_cursor_char() {
        assert_eq!(
            InputAction::from_key(Key::Left(KeyModifiers::NONE), &()),
            Some(InputAction::MoveCursor(
                Magnitude::Char,
                LineDirection::Left
            ))
        );
        assert_eq!(
            InputAction::from_key(Key::Right(KeyModifiers::NONE), &()),
            Some(InputAction::MoveCursor(
                Magnitude::Char,
                LineDirection::Right
            ))
        );
    }

    #[test]
    fn vertical_arrows_do_nothing() {
        assert_eq!(
            InputAction::from_key(Key::Up(KeyModifiers::NONE), &()),
            None
        );
        assert_eq!(
            InputAction::from_key(Key::Down(KeyModifiers::NONE), &()),
            None
        );
    }

    #[test]
    fn home_moves_to_beginning_of_line() {
        assert_eq!(
            InputAction::from_key(Key::Home, &()),
            Some(InputAction::MoveCursor(
                Magnitude::Line,
                LineDirection::Left
            ))
        );
    }

    #[test]
    fn end_moves_to_end_of_line() {
        assert_eq!(
            InputAction::from_key(Key::End, &()),
            Some(InputAction::MoveCursor(
                Magnitude::Line,
                LineDirection::Right
            ))
        );
    }

    #[test]
    fn arrows_with_ctrl_move_by_word() {
        assert_eq!(
            InputAction::from_key(Key::Left(KeyModifiers::CONTROL), &()),
            Some(InputAction::MoveCursor(
                Magnitude::Word,
                LineDirection::Left
            ))
        );
        assert_eq!(
            InputAction::from_key(Key::Right(KeyModifiers::CONTROL), &()),
            Some(InputAction::MoveCursor(
                Magnitude::Word,
                LineDirection::Right
            ))
        );
    }

    #[test]
    fn chars_generate_write_actions() {
        assert_eq!(
            InputAction::from_key(Key::Char('a', KeyModifiers::NONE), &()),
            Some(InputAction::Write('a'))
        );
        assert_eq!(
            InputAction::from_key(Key::Char('a', KeyModifiers::SHIFT), &()),
            Some(InputAction::Write('a'))
        );
        assert_eq!(
            InputAction::from_key(Key::Char('∑', KeyModifiers::NONE), &()),
            Some(InputAction::Write('∑'))
        );
        assert_eq!(
            InputAction::from_key(Key::Char('ã', KeyModifiers::SHIFT), &()),
            Some(InputAction::Write('ã'))
        );
        assert_eq!(
            InputAction::from_key(Key::Char('❤', KeyModifiers::SHIFT), &()),
            Some(InputAction::Write('❤'))
        );
        assert_eq!(
            InputAction::from_key(Key::Char('ç', KeyModifiers::SHIFT), &()),
            Some(InputAction::Write('ç'))
        );
        assert_eq!(
            InputAction::from_key(Key::Char('ñ', KeyModifiers::SHIFT), &()),
            Some(InputAction::Write('ñ'))
        );
    }

    #[test]
    fn page_up_and_down_do_nothing() {
        assert_eq!(
            InputAction::from_key(Key::PageUp(KeyModifiers::NONE), &()),
            None
        );
        assert_eq!(
            InputAction::from_key(Key::PageDown(KeyModifiers::NONE), &()),
            None
        );
    }
}
