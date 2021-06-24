pub(in crate) mod confirm;
pub(in crate) mod input;
pub(in crate) mod multiselect;
pub(in crate) mod password;
pub(in crate) mod select;

use std::error::Error;

pub use confirm::ConfirmOptions;
pub use input::InputOptions;
pub use multiselect::MultiSelectOptions;
pub use password::PasswordOptions;
pub use select::SelectOptions;

use crate::{renderer::Renderer, Answer};

pub(in crate) trait Prompt {
    fn prompt(self, renderer: &mut Renderer) -> Result<Answer, Box<dyn Error>>;
}
