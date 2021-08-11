//! UI-related definitions for rendered content.

mod backend;
mod color;
mod color_theme;
mod key;
mod old_terminal;
mod renderer;
mod style;

pub(in crate) use backend::*;
pub(in crate) use key::{Key, KeyModifiers};
pub(in crate) use old_terminal::OldTerminal;
pub(in crate) use renderer::Renderer;
pub(in crate) use style::Styled;

pub use color::Color;
pub use color_theme::ColorTheme;
pub use style::{Attributes, StyleSheet};
