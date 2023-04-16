use crate::Select;

#[derive(Copy, Clone, Debug)]
pub struct SelectConfig {
    pub vim_mode: bool,
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
