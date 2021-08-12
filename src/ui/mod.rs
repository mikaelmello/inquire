//! UI-related definitions for rendered content.

mod backend;
mod color;
mod color_theme;
mod key;
mod style;
mod terminal;

pub(in crate) use backend::*;
pub(in crate) use key::{Key, KeyModifiers};
pub(in crate) use style::Styled;
pub(in crate) use terminal::*;

pub use color::Color;
pub use color_theme::ColorTheme;
pub use style::{Attributes, StyleSheet};
