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

/// Mechanism to implement autocompletion features for text inputs. The `Autocomplete` trait has two provided methods: `get_suggestions` and `get_completion`.
///
/// - `get_suggestions` is called whenever the user's text input is modified, e.g. a new letter is typed, returning a `Vec<String>`. The `Vec<String>` is the list of suggestions that the prompt displays to the user according to their text input. The user can then navigate through the list and if they submit while highlighting one of these suggestions, the suggestion is treated as the final answer.
/// - `get_completion` is called whenever the user presses the autocompletion hotkey (`tab` by default), with the current text input and the text of the currently highlighted suggestion, if any, as parameters. This method should return whether any text replacement (an autocompletion) should be made. If the prompt receives a replacement to be made, it substitutes the current text input for the string received from the `get_completion` call.
///
/// For example, in the `complex_autocompletion.rs` example file, the `FilePathCompleter` scans the file system based on the current text input, storing a list of paths that match the current text input.
///
/// Every time `get_suggestions` is called, the method returns the list of paths that match the user input. When the user presses the autocompletion hotkey, the `FilePathCompleter` checks whether there is any path selected from the list, if there is, it decides to replace the current text input for it. The interesting piece of functionality is that if there isn't a path selected from the list, the `FilePathCompleter` calculates the longest common prefix amongst all scanned paths and updates the text input to an unambiguous new value. Similar to how terminals work when traversing paths.
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
