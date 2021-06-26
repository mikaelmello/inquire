pub(in crate) mod confirm;
pub(in crate) mod multiselect;
pub(in crate) mod password;
pub(in crate) mod select;
pub mod text;

use std::error::Error;

pub use confirm::Confirm;
pub use multiselect::MultiSelect;
pub use password::Password;
pub use select::Select;
pub use text::Text;

use crate::{renderer::Renderer, Answer};

pub(in crate) trait Prompt {
    fn prompt(self, renderer: &mut Renderer) -> Result<Answer, Box<dyn Error>>;
}
