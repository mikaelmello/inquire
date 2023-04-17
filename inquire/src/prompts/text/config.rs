use crate::Text;

/// Configuration settings used in the execution of a TextPrompt.
#[derive(Copy, Clone, Debug)]
pub struct TextConfig {
    /// Page size of the suggestion list, if it exists.
    pub page_size: usize,
}

impl From<&Text<'_>> for TextConfig {
    fn from(value: &Text<'_>) -> Self {
        Self {
            page_size: value.page_size,
        }
    }
}
