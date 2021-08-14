//! UI-related definitions for rendered content.

mod backend;
mod color;
mod key;
mod render_config;
mod style;
mod terminal;

pub(in crate) use backend::*;
pub(in crate) use key::{Key, KeyModifiers};
pub(in crate) use terminal::*;

pub use color::Color;
pub use render_config::*;
pub use style::{Attributes, StyleSheet, Styled};
