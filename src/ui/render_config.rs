use super::{Color, StyleSheet};

/// Color theme that can be applied to a prompt.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RenderConfig {
    /// Cursor in text inputs.
    pub cursor: StyleSheet,
}

impl RenderConfig {
    /// RenderConfig in which no colors or attributes are applied.
    pub fn empty() -> Self {
        Self {
            cursor: StyleSheet::empty(),
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            cursor: StyleSheet::empty()
                .with_bg(Color::Grey)
                .with_fg(Color::Black),
        }
    }
}
