use dyn_clone::DynClone;

use crate::CustomUserError;

pub trait AutoComplete: DynClone {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError>;

    fn get_suggestions(&self) -> Result<Vec<String>, CustomUserError>;

    fn get_completion(
        &self,
        selected_suggestion: Option<(usize, &str)>,
    ) -> Result<Completion, CustomUserError>;
}

pub enum Completion {
    Replace(String),
    Append(String),
    None,
}

impl Clone for Box<dyn AutoComplete> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}
