use crate::PathSelect;

/// Configuration settings used in the execution of a PathSelectPrompt.
#[derive(Copy, Clone, Debug)]
pub struct PathSelectConfig {
    /// Whether to use vim-style keybindings (unsupported).
    pub vim_mode: bool,
    /// Page size of the list of options.
    pub page_size: usize,
    /// Whether to keep the filter text when an option is selected.
    pub keep_filter: bool,
}

impl<T> From<&PathSelect<'_, T>> for PathSelectConfig {
    fn from(value: &PathSelect<'_, T>) -> Self {
        Self {
            vim_mode: value.vim_mode,
            page_size: value.page_size,
            keep_filter: value.keep_filter,
        }
    }
}
