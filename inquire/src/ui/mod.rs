//! UI-related definitions for rendered content.

mod backend;
mod color;
pub(crate) mod dimension;
mod input_reader;
mod key;
mod render_config;
mod style;
mod untitled_render_box_abstraction;

pub(crate) use backend::*;
pub(crate) use input_reader::*;
pub(crate) use key::*;

pub use color::Color;
pub use render_config::*;
pub use style::{Attributes, StyleSheet, Styled};
