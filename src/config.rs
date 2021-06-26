pub type Filter = fn(filter: &str, value: &str, index: usize) -> bool;
pub type Suggestor = fn(value: &str) -> Vec<String>;

pub const DEFAULT_PAGE_SIZE: usize = 7;
pub const DEFAULT_VIM_MODE: bool = false;

pub const DEFAULT_FILTER: Filter = |filter: &str, value: &str, _| -> bool {
    let filter = filter.to_lowercase();

    value.to_lowercase().contains(&filter)
};
