use crate::Select;

/// Configuration settings used in the execution of a SelectPrompt.
#[derive(Copy, Clone, Debug)]
pub struct SelectConfig {
    /// Whether to use vim-style keybindings.
    pub vim_mode: bool,
    /// Page size of the list of options.
    pub page_size: usize,
}

impl<T> From<&Select<'_, T>> for SelectConfig {
    fn from(value: &Select<'_, T>) -> Self {
        Self {
            vim_mode: value.vim_mode,
            page_size: value.page_size,
        }
    }
}
