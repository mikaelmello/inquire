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

impl InputAction {
    /// Generates a list of `Write(char)` actions with the contents
    /// of a source string.
    #[allow(unused)]
    fn gen_write_from_str(value: &str) -> Vec<InputAction> {
        value.chars().map(InputAction::Write).collect()
    }
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
