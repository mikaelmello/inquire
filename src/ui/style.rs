use bitflags::bitflags;

use super::Color;

bitflags! {
    /// Attributes to apply to a text via the [StyleSheet] struct.
    ///
    /// These attributes are flags and can thus be combined.
    ///
    /// # Example
    ///
    /// ```
    /// use inquire::ui::Attributes;
    ///
    /// let attributes = Attributes::ITALIC | Attributes::BOLD;
    ///
    /// assert!(attributes.contains(Attributes::BOLD));
    /// assert!(attributes.contains(Attributes::ITALIC));
    /// ```
    pub struct Attributes: u8 {
        /// Increases the text intensity
        const BOLD   = 0b01;

        /// Emphasises the text.
        const ITALIC = 0b10;
    }
}

/// Style definitions that can be applied to the rendered content.
///
/// # Example
///
/// ```
//. use inquire::ui::{StyleSheet, Color, Attributes};
///
/// let style_sheet = StyleSheet::default();
///
/// assert!(style_sheet.is_empty());
///
/// let style_sheet = style_sheet
///     .with_bg(Color::Blue)
///     .with_attr(Attributes::ITALIC | Attributes::BOLD);
///
/// assert!(!style_sheet.is_empty());
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StyleSheet {
    /// Foreground color of text.
    pub fg: Option<Color>,
    /// Background color of text.
    pub bg: Option<Color>,
    /// Attributes applied to text.
    pub att: Attributes,
}

impl StyleSheet {
    /// A stylesheet with no colors and no attributes.
    pub fn empty() -> Self {
        Self {
            fg: None,
            bg: None,
            att: Attributes::empty(),
        }
    }

    /// Check if the stylesheet contains no colors and no attributes.
    pub fn is_empty(&self) -> bool {
        self.fg.is_none() && self.bg.is_none() && self.att.is_empty()
    }

    /// Copies the StyleSheet to a new one set with the defined foreground [Color].
    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    /// Copies the StyleSheet to a new one set with the defined background [Color].
    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    /// Applies the defined attributes to the style sheet.
    ///
    /// Warning: this does not keep the previously applied attributes. If you want
    /// to just set a new attribute and keep the others, you need to apply the OR
    /// operation yourself.
    ///
    /// # Example
    ///
    /// ```
    /// use inquire::ui::{StyleSheet, Attributes};
    ///
    /// let style_sheet = StyleSheet::default().with_attr(Attributes::BOLD);
    /// assert_eq!(true,  style_sheet.att.contains(Attributes::BOLD));
    /// assert_eq!(false, style_sheet.att.contains(Attributes::ITALIC));
    ///
    /// let style_sheet = style_sheet.with_attr(Attributes::ITALIC);
    /// assert_eq!(false, style_sheet.att.contains(Attributes::BOLD));
    /// assert_eq!(true,  style_sheet.att.contains(Attributes::ITALIC));
    ///
    /// let style_sheet = style_sheet.with_attr(style_sheet.att | Attributes::BOLD);
    /// assert_eq!(true, style_sheet.att.contains(Attributes::BOLD));
    /// assert_eq!(true, style_sheet.att.contains(Attributes::ITALIC));
    /// ```
    pub fn with_attr(mut self, attributes: Attributes) -> Self {
        self.att = attributes;
        self
    }
}

impl Default for StyleSheet {
    /// A stylesheet with no colors and no attributes.
    fn default() -> Self {
        Self::empty()
    }
}

pub struct Styled<T> {
    pub content: T,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub att: Attributes,
}

impl<T> Styled<T> {
    pub fn new(content: T) -> Self {
        Self {
            content,
            fg: None,
            bg: None,
            att: Attributes::empty(),
        }
    }

    pub fn with_style_sheet(mut self, stylesheet: StyleSheet) -> Self {
        self.fg = stylesheet.fg;
        self.bg = stylesheet.bg;
        self.att = stylesheet.att;
        self
    }

    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    #[allow(unused)]
    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn with_attr(mut self, attributes: Attributes) -> Self {
        self.att = self.att | attributes;
        self
    }

    pub fn reset_attr(mut self) -> Self {
        self.att = Attributes::empty();
        self
    }
}
