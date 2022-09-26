use dyn_clone::DynClone;

use crate::CustomUserError;

/// Used when an auto-completion is triggered for the user's text input.
///
/// `None` means that no completion will be made.
/// `Some(String)` will replace the current text input with the `String` in `Some`.
pub type Replacement = Option<String>;

pub trait AutoComplete: DynClone {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError>;

    fn get_suggestions(&self) -> Result<Vec<String>, CustomUserError>;

    fn get_completion(
        &self,
        selected_suggestion: Option<(usize, &str)>,
    ) -> Result<Replacement, CustomUserError>;
}

impl Clone for Box<dyn AutoComplete> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}
