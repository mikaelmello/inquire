use bitflags::bitflags;

bitflags! {
    pub struct Attributes: u8 {
        const BOLD              = 0b01;
        const ITALIC            = 0b10;
    }
}

/// Re-export of Crossterm API
pub type Color = crossterm::style::Color;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StyleSheet {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub attributes: Attributes,
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            attributes: Attributes::empty(),
        }
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
        self.att = stylesheet.attributes;
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
