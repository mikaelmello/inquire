//! Contains definitions to apply style to rendered contents.

use std::fmt::Display;

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
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
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
/// use inquire::ui::{Attributes, Color, StyleSheet};
///
/// let style_sheet = StyleSheet::default();
///
/// assert!(style_sheet.is_empty());
///
/// let style_sheet = style_sheet
///     .with_bg(Color::DarkBlue)
///     .with_attr(Attributes::ITALIC | Attributes::BOLD);
///
/// assert!(!style_sheet.is_empty());
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StyleSheet {
    /// Foreground color of text.
    pub fg: Option<Color>,
    /// Background color of text.
    pub bg: Option<Color>,
    /// Attributes applied to text.
    pub att: Attributes,
}

impl StyleSheet {
    /// Creates a style sheet with no colors and no attributes
    pub fn new() -> Self {
        Self::empty()
    }

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

    /// Copies the style sheet to a new one with the specified attributes.
    ///
    /// Warning: this does not keep the previously applied attributes. If you want
    /// to just set a new attribute and keep the others, you need to apply the OR
    /// operation yourself.
    ///
    /// # Example
    ///
    /// ```
    /// use inquire::ui::{Attributes, Color, StyleSheet};
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

/// Represents a content that when rendered must have the associated style
/// applied to it.
#[derive(Clone, Debug)]
pub struct Styled<T>
where
    T: Display,
{
    /// Content to be rendered.
    pub content: T,

    /// Style sheet to be applied to content when rendered.
    pub style: StyleSheet,
}

impl<T> Styled<T>
where
    T: Display,
{
    /// Creates a new `Styled` object with the specified content
    /// and a default (empty) style sheet.
    pub fn new(content: T) -> Self {
        Self {
            content,
            style: StyleSheet::default(),
        }
    }

    /// Sets the style sheet to the styled struct.
    #[allow(unused)]
    pub fn with_style_sheet(mut self, style_sheet: StyleSheet) -> Self {
        self.style = style_sheet;
        self
    }

    /// Sets the styled content to have the defined foreground [Color].
    pub fn with_fg(mut self, fg: Color) -> Self {
        self.style.fg = Some(fg);
        self
    }

    /// Sets the styled content to have the defined foreground [Color].
    pub fn with_bg(mut self, bg: Color) -> Self {
        self.style.bg = Some(bg);
        self
    }

    /// Sets the styled content to have the defined attributes.
    ///
    /// Warning: this does not keep the previously applied attributes. If you want
    /// to just set a new attribute and keep the others, you need to apply the OR
    /// operation yourself.
    pub fn with_attr(mut self, attributes: Attributes) -> Self {
        self.style.att = attributes;
        self
    }

    /// Updates the content while keeping the style sheet constant.
    pub fn with_content<U>(self, content: U) -> Styled<U>
    where
        U: Display,
    {
        Styled {
            content,
            style: self.style,
        }
    }
}

impl<T> Copy for Styled<T> where T: Copy + Display {}

impl<T> Default for Styled<T>
where
    T: Default + Display,
{
    fn default() -> Self {
        Self {
            content: Default::default(),
            style: Default::default(),
        }
    }
}

impl<T> From<T> for Styled<T>
where
    T: Display,
{
    fn from(from: T) -> Self {
        Self::new(from)
    }
}
