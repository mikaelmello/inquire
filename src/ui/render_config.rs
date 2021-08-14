use lazy_static::lazy_static;

use super::{Color, StyleSheet, Styled};

/// Color theme that can be applied to a prompt.
#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// Prefix added before prompts.
    ///
    /// Note: a space character will be added to separate the prefix
    /// and the prompt message.
    pub prompt_prefix: Styled<&'static str>,

    /// Style of the prompt message, applicable to all prompt types.
    pub prompt: StyleSheet,

    /// Cursor in text inputs.
    pub cursor: StyleSheet,
}

impl RenderConfig {
    /// RenderConfig in which no colors or attributes are applied.
    pub fn empty() -> Self {
        Self {
            prompt_prefix: Styled::new("?"),
            prompt: StyleSheet::empty(),
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

    /// Sets the prompt prefix.
    pub fn with_prompt_prefix(mut self, prompt_prefix: Styled<&'static str>) -> Self {
        self.prompt_prefix = prompt_prefix;
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            prompt_prefix: Styled::new("?").with_fg(Color::Green),
            prompt: StyleSheet::empty(),
            cursor: StyleSheet::empty()
                .with_bg(Color::Grey)
                .with_fg(Color::Black),
        }
    }
}
