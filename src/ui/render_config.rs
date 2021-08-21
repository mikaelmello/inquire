use lazy_static::lazy_static;

use super::{Color, StyleSheet, Styled};

/// Rendering configuration that can be applied to a prompt.
///
/// Render configurations can set mostly style sheets for particular
/// parts of the prompt layout. Additionally, it allows you to set
/// the content of a few tokens, such as prompt or error message prefixes.
///
/// # Example
///
/// ```
/// use inquire::ui::{Color, RenderConfig, Styled};
///
/// let empty: RenderConfig = RenderConfig::empty();
/// let default: RenderConfig = RenderConfig::default();
///
/// let default_used_in_prompts: &'static RenderConfig = RenderConfig::default_static_ref();
///
/// let prompt_prefix = Styled::new("$").with_fg(Color::Red);
/// let mine = default.with_prompt_prefix(prompt_prefix);
/// ```
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

    /// Render configuration of placeholders.
    ///
    /// Note: placeholders are displayed wrapped in parenthesis, e.g. (yes).
    /// Non-styled space characters is added before the default value display
    /// and after the default value, as separators.
    pub placeholder: StyleSheet,

    /// Render configuration of placeholder cursors.
    ///
    /// Note: placeholders are displayed wrapped in parenthesis, e.g. (yes).
    /// Non-styled space characters is added before the default value display
    /// and after the default value, as separators.
    pub placeholder_cursor: StyleSheet,

    /// Render configuration of help messages.
    ///
    /// Note: help messages are displayed wrapped in brackets, e.g. [Be careful!].
    pub help_message: StyleSheet,

    /// Character used to mask password text inputs when in mode
    /// [`Masked`](crate::prompts::PasswordDisplayMode).
    ///
    /// Note: Styles for masked text inputs are set in the
    /// [`text_input`](crate::ui::RenderConfig::text_input) configuration.
    pub password_mask: char,

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

    /// Render configuration for error messages.
    pub error_message: ErrorMessageRenderConfig,

    /// Prefix for options.
    ///
    /// Note: a space character will be added to separate the prefix
    /// and the option value or the checkbox.
    pub option_prefix: Styled<&'static str>,

    /// Selected checkbox in multi-select options.
    ///
    /// Note: a space character will be added to separate the checkbox
    /// from a possible prefix, and to separate the checkbox from the
    /// option value to the right.
    pub selected_checkbox: Styled<&'static str>,

    /// Unselected checkbox in multi-select options.
    ///
    /// Note: a space character will be added to separate the checkbox
    /// from a possible prefix, and to separate the checkbox from the
    /// option value to the right.
    pub unselected_checkbox: Styled<&'static str>,

    /// Style sheet for options.
    ///
    /// Note: a non-styled space character is added before the option value as
    /// a separator from the prefix.
    pub option: StyleSheet,

    /// Render configuration for calendar

    #[cfg(feature = "date")]
    pub calendar: calendar::CalendarRenderConfig,
}

impl RenderConfig {
    /// RenderConfig in which no colors or attributes are applied.
    pub fn empty() -> Self {
        Self {
            prompt_prefix: Styled::new("?"),
            prompt: StyleSheet::empty(),
            default_value: StyleSheet::empty(),
            placeholder: StyleSheet::empty(),
            placeholder_cursor: StyleSheet::empty(),
            help_message: StyleSheet::empty(),
            text_input: InputRenderConfig::empty(),
            error_message: ErrorMessageRenderConfig::empty(),
            answer: StyleSheet::empty(),
            password_mask: '*',
            option_prefix: Styled::new(">"),
            selected_checkbox: Styled::new("[x]"),
            unselected_checkbox: Styled::new("[ ]"),
            option: StyleSheet::empty(),

            #[cfg(feature = "date")]
            calendar: calendar::CalendarRenderConfig::empty(),
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

    /// Sets the prompt prefix and its style sheet.
    pub fn with_prompt_prefix(mut self, prompt_prefix: Styled<&'static str>) -> Self {
        self.prompt_prefix = prompt_prefix;
        self
    }

    /// Sets the text input render configuration.
    pub fn with_text_input(mut self, text_input: InputRenderConfig) -> Self {
        self.text_input = text_input;
        self
    }

    /// Sets the style sheet for default values.
    pub fn with_default_value(mut self, default_value: StyleSheet) -> Self {
        self.default_value = default_value;
        self
    }

    /// Sets the style sheet for help messages.
    pub fn with_help_message(mut self, help_message: StyleSheet) -> Self {
        self.help_message = help_message;
        self
    }

    /// Sets the style sheet for answers.
    pub fn with_answer(mut self, answer: StyleSheet) -> Self {
        self.answer = answer;
        self
    }

    /// Sets the render configuration for error messages.
    pub fn with_error_message(mut self, error_message: ErrorMessageRenderConfig) -> Self {
        self.error_message = error_message;
        self
    }

    /// Sets the styled component for option prefixes.
    pub fn with_option_prefix(mut self, option_prefix: Styled<&'static str>) -> Self {
        self.option_prefix = option_prefix;
        self
    }

    /// Sets the styled component for selected checkboxes.
    pub fn with_selected_checkbox(mut self, selected_checkbox: Styled<&'static str>) -> Self {
        self.selected_checkbox = selected_checkbox;
        self
    }

    /// Sets the styled component for unselected checkboxes.
    pub fn with_unselected_checkbox(mut self, unselected_checkbox: Styled<&'static str>) -> Self {
        self.unselected_checkbox = unselected_checkbox;
        self
    }

    /// Sets the style sheet for option values.
    pub fn with_option(mut self, option: StyleSheet) -> Self {
        self.option = option;
        self
    }

    #[cfg(feature = "date")]
    /// Sets the render configuration for calendars.
    pub fn with_calendar_config(mut self, calendar: calendar::CalendarRenderConfig) -> Self {
        self.calendar = calendar;
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            prompt_prefix: Styled::new("?").with_fg(Color::Green),
            prompt: StyleSheet::empty(),
            default_value: StyleSheet::empty(),
            placeholder: StyleSheet::new().with_fg(Color::DarkGrey),
            placeholder_cursor: StyleSheet::new()
                .with_fg(Color::Black)
                .with_bg(Color::DarkGrey),
            help_message: StyleSheet::empty().with_fg(Color::Cyan),
            text_input: InputRenderConfig::default(),
            error_message: ErrorMessageRenderConfig::default(),
            password_mask: '*',
            answer: StyleSheet::empty().with_fg(Color::Cyan),
            option_prefix: Styled::new(">").with_fg(Color::Cyan),
            selected_checkbox: Styled::new("[x]").with_fg(Color::Green),
            unselected_checkbox: Styled::new("[ ]"),
            option: StyleSheet::empty(),

            #[cfg(feature = "date")]
            calendar: calendar::CalendarRenderConfig::default(),
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

/// Render configuration for error messages.
#[derive(Clone, Debug)]
pub struct ErrorMessageRenderConfig {
    /// Prefix style.
    pub prefix: Styled<&'static str>,

    /// Separator style.
    ///
    /// Note: This separator is a space character. It might be useful to
    /// style it if you want to set a background color for error messages.
    pub separator: StyleSheet,

    /// Message style.
    pub message: StyleSheet,
}

impl ErrorMessageRenderConfig {
    /// Render configuration in which no colors or attributes are applied.
    pub fn empty() -> Self {
        Self {
            prefix: Styled::new("#"),
            separator: StyleSheet::empty(),
            message: StyleSheet::empty(),
        }
    }

    /// Sets the prefix.
    pub fn with_prefix(mut self, prefix: Styled<&'static str>) -> Self {
        self.prefix = prefix;
        self
    }

    /// Sets the separator stylesheet.
    ///
    /// Note: This separator is a space character. It might be useful to
    /// style it if you want to set a background color for error messages.
    pub fn with_separator(mut self, separator: StyleSheet) -> Self {
        self.separator = separator;
        self
    }

    /// Sets the message stylesheet.
    pub fn with_message(mut self, message: StyleSheet) -> Self {
        self.message = message;
        self
    }
}

impl Default for ErrorMessageRenderConfig {
    fn default() -> Self {
        Self {
            prefix: Styled::new("#").with_fg(Color::Red),
            separator: StyleSheet::empty(),
            message: StyleSheet::empty().with_fg(Color::Red),
        }
    }
}

#[cfg(feature = "date")]
mod calendar {
    use super::{Color, StyleSheet, Styled};

    /// Calendar configuration for error messages.
    #[derive(Clone, Debug)]
    pub struct CalendarRenderConfig {
        /// Prefix style.
        pub prefix: Styled<&'static str>,

        /// Style sheet for the calendar header, e.g. january 2021.
        pub header: StyleSheet,

        /// Style sheet for the calendar week header, e.g. su mo tu we th fr sa.
        pub week_header: StyleSheet,

        /// Style sheet for the currently selected date.
        pub selected_date: StyleSheet,

        /// Style sheet for today's date, just for hinting purposes.
        pub today_date: StyleSheet,

        /// Style sheet for dates that are from the previous or next month
        /// displayed in the calendar.
        pub different_month_date: StyleSheet,

        /// Style sheet for dates that can not be selected due to the
        /// min/max settings.
        pub unavailable_date: StyleSheet,
    }

    impl CalendarRenderConfig {
        /// Render configuration in which no colors or attributes are applied.
        pub fn empty() -> Self {
            Self {
                prefix: Styled::new(">"),
                header: StyleSheet::empty(),
                week_header: StyleSheet::empty(),
                selected_date: StyleSheet::empty(),
                today_date: StyleSheet::empty(),
                different_month_date: StyleSheet::empty(),
                unavailable_date: StyleSheet::empty(),
            }
        }

        /// Sets the prefix.
        pub fn with_prefix(mut self, prefix: Styled<&'static str>) -> Self {
            self.prefix = prefix;
            self
        }
    }

    impl Default for CalendarRenderConfig {
        fn default() -> Self {
            Self {
                prefix: Styled::new(">").with_fg(Color::Green),
                header: StyleSheet::empty(),
                week_header: StyleSheet::empty(),
                selected_date: StyleSheet::empty()
                    .with_fg(Color::Black)
                    .with_bg(Color::Grey),
                today_date: StyleSheet::empty().with_fg(Color::Green),
                different_month_date: StyleSheet::empty().with_fg(Color::DarkGrey),
                unavailable_date: StyleSheet::empty().with_fg(Color::DarkGrey),
            }
        }
    }
}
