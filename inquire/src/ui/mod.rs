//! UI-related definitions for rendered content.

mod backend;
mod color;
mod input_reader;
mod key;
mod render_config;
mod renderer;
mod style;

pub(crate) use backend::*;
pub(crate) use input_reader::*;
pub(crate) use key::*;
pub(crate) use renderer::*;

pub use color::Color;
pub use render_config::*;
pub use style::{Attributes, StyleSheet, Styled};
