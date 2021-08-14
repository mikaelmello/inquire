use lazy_static::lazy_static;

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

    /// Static reference to a [default](crate::ui::RenderConfig::default) render configuration.
    pub fn default_static_ref() -> &'static Self {
        lazy_static! {
            static ref DEFAULT_RENDER_CONFIG: RenderConfig = RenderConfig::default();
        };

        &DEFAULT_RENDER_CONFIG
    }

    /// Static reference to an [empty](crate::ui::RenderConfig::empty) render configuration.
    pub fn empty_static_ref() -> &'static Self {
        lazy_static! {
            static ref EMPTY_RENDER_CONFIG: RenderConfig = RenderConfig::empty();
        };

        &EMPTY_RENDER_CONFIG
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
