//! UI-related definitions for rendered content.

mod backend;
mod color;
pub(crate) mod dimension;
mod frame_renderer;
mod input_reader;
mod key;
mod render_config;
mod style;

pub(crate) use backend::*;
pub(crate) use input_reader::*;
pub(crate) use key::*;

pub use color::Color;
pub use render_config::*;
pub use style::{Attributes, StyleSheet, Styled};
