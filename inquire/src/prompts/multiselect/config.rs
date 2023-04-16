use crate::MultiSelect;

#[derive(Copy, Clone, Debug)]
pub struct MultiSelectConfig {
    pub vim_mode: bool,
    pub page_size: usize,
    pub keep_filter: bool,
}

impl<T> From<&MultiSelect<'_, T>> for MultiSelectConfig {
    fn from(value: &MultiSelect<'_, T>) -> Self {
        Self {
            vim_mode: value.vim_mode,
            page_size: value.page_size,
            keep_filter: value.keep_filter,
        }
    }
}
