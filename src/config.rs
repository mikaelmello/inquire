//! Module containing general type aliases and default values used by multiple prompt types.

/// Type alias to represent the function used to filter options.
/// The function receives the current filter value and the value and index values of the option.
/// The return type should be whether the current option should be displayed to the user.
pub type Filter<'a> = &'a dyn Fn(&str, &str, usize) -> bool;

/// Type alias to represent the function used to retrieve text input suggestions.
/// The function receives the current input and should return a collection of strings
/// containing the suggestions to be made to the user.
pub type Suggester<'a> = &'a dyn Fn(&str) -> Vec<String>;

/// Default page size when displaying options to the user.
pub const DEFAULT_PAGE_SIZE: usize = 7;

/// Default value of vim mode.
pub const DEFAULT_VIM_MODE: bool = false;

/// Default filter function, which checks if the current filter value is a substring of the option value.
/// If it is, the option is displayed.
pub const DEFAULT_FILTER: Filter = &|filter, value, _| -> bool {
    let filter = filter.to_lowercase();

    value.to_lowercase().contains(&filter)
};
