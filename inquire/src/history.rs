//! Trait and structs used by prompts to provide prompt-entry history navigation features.
//!
//! History is triggered upon up or down keystroke, which iterates through a historical
//! list of prompt entries. Each keystroke rewrites the contents of the prompt with the next
//! item from that history.

use dyn_clone::DynClone;

use crate::CustomUserError;

/// Used when iterating through the user's prompt history
///
/// `None` means that no completion will be made.
/// `Some(String)` will replace the current text input with the `String` in `Some`.
pub type Replacement = Option<String>;

/// Mechanism to implement history navigation
///
/// - `earlier_element` is called whenever the user presses the `up` key, and navigates towards less recent prompt entries
/// - `later_element` is called whenever the user presses the `down` key, and navigates towards more recent prompt entries
/// - `prepend_element` is called whenever the user hits `enter` commits a new entry into the prompt; this adds the entry to the prompt history
pub trait History: DynClone {
    /// Update internal state and return the next less-recent history entry
    fn earlier_element(&mut self) -> Result<Replacement, CustomUserError>;

    /// Update internal state and return the nezt more-recent history entry
    fn later_element(&mut self) -> Result<Replacement, CustomUserError>;

    /// Add the given string to the history, at the beginning of the history, which is the most recent
    fn prepend_element(&mut self, _: String);
}

impl Clone for Box<dyn History> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

/// Empty struct and implementation of History trait. Used for the default
/// history of `Text` prompts.
#[derive(Clone, Default)]
pub struct NoHistory;

impl History for NoHistory {
    fn earlier_element(&mut self) -> Result<Replacement, CustomUserError> {
        Ok(Replacement::None)
    }

    fn later_element(&mut self) -> Result<Replacement, CustomUserError> {
        Ok(Replacement::None)
    }

    fn prepend_element(&mut self, _: String) {
        ()
    }
}

/// Simple `History` implementation. Stores a vector of strings representing prompt entries, and navigates through the
/// vector. Prepends new entries into the vector and resets internal index state.
#[derive(Clone, Debug)]
pub struct SimpleHistory {
    history: Vec<String>,
    index: isize,
}

impl SimpleHistory {
    /// Reeturn a new SimpleHistory, bootstrappd with the given entries
    pub fn new(initial_history: Vec<String>) -> Self {
        SimpleHistory {
            history: initial_history,
            index: -1,
        }
    }
}

impl History for SimpleHistory {
    fn earlier_element(&mut self) -> Result<Replacement, CustomUserError> {
        if self.index < self.history.len() as isize - 1 {
            self.index += 1;
            Ok(Replacement::Some(self.history[self.index as usize].clone()))
        } else if !self.history.is_empty() && self.index == self.history.len() as isize - 1 {
            // return last item in the history
            Ok(Replacement::Some(self.history[self.index as usize].clone()))
        } else {
            Ok(Replacement::None)
        }
    }

    fn later_element(&mut self) -> Result<Replacement, CustomUserError> {
        if self.index >= 1 {
            self.index -= 1;
            let e = self.history[self.index as usize].clone();
            Ok(Replacement::Some(e))
        } else {
            if self.index == 0 {
                self.index -= 1;
            }

            Ok(Replacement::None)
        }
    }

    fn prepend_element(&mut self, e: String) {
        if e.len() > 0 {
            self.history.insert(0, e);
            self.index = -1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history() {
        let history = SimpleHistory::new(vec!["first".into(), "second".into()]);
        assert_eq!(history.history.len(), 2);
        assert_eq!(history.index, -1);
    }

    #[test]
    fn test_prepend_element() {
        let mut history = SimpleHistory::new(vec![]);
        history.prepend_element("new".into());
        assert_eq!(history.history.len(), 1);
        assert_eq!(history.history[0], "new");
    }

    #[test]
    fn test_earlier_then_later_element() {
        let mut history = SimpleHistory::new(vec!["first".into(), "second".into()]);
        assert_eq!(
            history.earlier_element().unwrap(),
            Replacement::Some("first".into())
        );
        assert_eq!(history.later_element().unwrap(), Replacement::None);
        assert_eq!(
            history.earlier_element().unwrap(),
            Replacement::Some("first".into())
        );
    }

    #[test]
    fn test_sequence_of_operations() {
        let mut history = SimpleHistory::new(vec!["first".into(), "second".into()]);
        // Prepend a new element
        history.prepend_element("zero".into());
        // Move to the earlier element, which should now be "zero"
        assert_eq!(
            history.earlier_element().unwrap(),
            Replacement::Some("zero".into())
        );
        // Move to an even earlier element, which should now be "second"
        assert_eq!(
            history.earlier_element().unwrap(),
            Replacement::Some("first".into())
        );
        // Try to move later, back to "first"
        assert_eq!(
            history.later_element().unwrap(),
            Replacement::Some("zero".into())
        );
        // And now, since we're at the start, it should return None
        assert_eq!(history.later_element().unwrap(), Replacement::None);
    }

    #[test]
    fn test_no_prepend_of_empty_string() {
        let mut history = SimpleHistory::new(vec!["initial".into()]);
        history.prepend_element("".into());
        assert_eq!(history.history.len(), 1);
        assert_eq!(history.history[0], "initial");
    }
}
