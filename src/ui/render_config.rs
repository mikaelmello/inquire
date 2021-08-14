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

    /// Render configuration of default values.
    ///
    /// Note: default values are displayed wrapped in parenthesis, e.g. (yes).
    /// Non-styled space characters is added before the default value display
    /// and after the default value, as separators.
    pub default_value: StyleSheet,

    /// Render configuration of text inputs.
    ///
    /// Note: a non-styled space character is added before the text input as
    /// a separator from the prompt message (or default value display).
    pub text_input: InputRenderConfig,

    /// Render configuration of final prompt answers (submissions).
    ///
    /// Note: a non-styled space character is added before the answer as
    /// a separator from the prompt message (or default value display).
    pub answer: StyleSheet,
}

impl RenderConfig {
    /// RenderConfig in which no colors or attributes are applied.
    pub fn empty() -> Self {
        Self {
            prompt_prefix: Styled::new("?"),
            prompt: StyleSheet::empty(),
            default_value: StyleSheet::empty(),
            text_input: InputRenderConfig::empty(),
            answer: StyleSheet::empty(),
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

    /// Sets the prompt prefix.
    pub fn with_text_input(mut self, text_input: InputRenderConfig) -> Self {
        self.text_input = text_input;
        self
    }

    /// Sets the prompt prefix.
    pub fn with_text(mut self, text: StyleSheet) -> Self {
        self.text_input = self.text_input.with_text(text);
        self
    }

    /// Sets the prompt prefix.
    pub fn with_cursor(mut self, cursor: StyleSheet) -> Self {
        self.text_input = self.text_input.with_cursor(cursor);
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            prompt_prefix: Styled::new("?").with_fg(Color::Green),
            prompt: StyleSheet::empty(),
            default_value: StyleSheet::empty(),
            text_input: InputRenderConfig::default(),
            answer: StyleSheet::empty().with_fg(Color::Cyan),
        }
    }
}

/// Render configuration for text inputs.
///
/// All text will be rendered with the `text`
/// style sheet applied, except for the one character
/// behind the cursor, which will have the `cursor`
/// style sheet applied.
#[derive(Clone, Debug)]
pub struct InputRenderConfig {
    /// Text style.
    pub text: StyleSheet,

    /// Cursor style.
    pub cursor: StyleSheet,
}

impl InputRenderConfig {
    /// Render configuration in which no colors or attributes are applied.
    pub fn empty() -> Self {
        Self {
            text: StyleSheet::empty(),
            cursor: StyleSheet::empty(),
        }
    }

    /// Sets the text stylesheet.
    pub fn with_text(mut self, text: StyleSheet) -> Self {
        self.text = text;
        self
    }

    /// Sets the cursor stylesheet.
    pub fn with_cursor(mut self, cursor: StyleSheet) -> Self {
        self.cursor = cursor;
        self
    }
}

impl Default for InputRenderConfig {
    fn default() -> Self {
        Self {
            text: StyleSheet::empty(),
            cursor: StyleSheet::empty()
                .with_bg(Color::Grey)
                .with_fg(Color::Black),
        }
    }
}
