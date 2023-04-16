use crate::Text;

#[derive(Copy, Clone, Debug)]
pub struct TextConfig {
    pub page_size: usize,
}

impl From<&Text<'_>> for TextConfig {
    fn from(value: &Text<'_>) -> Self {
        Self {
            page_size: value.page_size,
        }
    }
}
