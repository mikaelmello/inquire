//! Trait and structs used by prompts to provide autocompletion features.
//!
//! Autocompleters receive the user input to a given prompt and may return
//! a list of suggestions, selectable by the user as an option to be completed.
//!
//! When the user presses the autocompletion hotkey (`tab` by default) the
//! autocompleter receives the current text input and the currently highlighted
//! selection, if any. Then the developer may return a [Replacement] action
//! where the current user text input is replaced or not by a provided string.
//!
//! Check the example files to see some usages, recommended are `expense_tracker.rs`
//! and `complex_autocompletion.rs`.

use dyn_clone::DynClone;

use crate::CustomUserError;

/// Used when an autocompletion is triggered for the user's text input.
///
/// `None` means that no completion will be made.
/// `Some(String)` will replace the current text input with the `String` in `Some`.
pub type Replacement = Option<String>;

/// Mechanism to implement autocompletion features for text inputs.
pub trait Autocomplete: DynClone {
    /// List of input suggestions to be displayed to the user upon typing the
    /// text input.
    ///
    /// If the user presses the autocompletion hotkey (`tab` as default) with
    /// a suggestion highlighted, the user's text input will be replaced by the
    /// content of the suggestion string.
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError>;

    /// Standalone autocompletion that can be implemented based solely on the user's
    /// input.
    ///
    /// If the user presses the autocompletion hotkey (`tab` as default) and
    /// there are no suggestions highlighted (1), this function will be called in an
    /// attempt to autocomplete the user's input.
    ///
    /// If the returned value is of the `Some` variant, the text input will be replaced
    /// by the content of the string.
    ///
    /// (1) This applies where either there are no suggestions at all, or there are
    /// some displayed but the user hasn't highlighted any.
    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError>;
}

impl Clone for Box<dyn Autocomplete> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

/// Empty struct and implementation of Autocomplete trait. Used for the default
/// autocompleter of `Text` prompts.
#[derive(Clone, Default)]
pub struct NoAutoCompletion;

impl Autocomplete for NoAutoCompletion {
    fn get_suggestions(&mut self, _: &str) -> Result<Vec<String>, CustomUserError> {
        Ok(vec![])
    }

    fn get_completion(
        &mut self,
        _: &str,
        _: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        Ok(Replacement::None)
    }
}

impl<F> Autocomplete for F
where
    F: Fn(&str) -> Result<Vec<String>, CustomUserError> + Clone,
{
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        (self)(input)
    }

    fn get_completion(
        &mut self,
        _: &str,
        suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        Ok(suggestion)
    }
}
