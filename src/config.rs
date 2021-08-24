//! General type aliases and default values used by multiple prompts.

/// Type alias to represent the function used to filter options.
///
/// The function receives:
/// - Current user input, filter value
/// - Current option being evaluated, with type preserved
/// - String value of the current option
/// - Index of the current option in the original list
///
/// The return type should be whether the current option should be displayed to the user.
///
/// # Examples
///
/// ```
/// use inquire::config::Filter;
///
/// let filter: Filter = &|filter, value, _| -> bool {
///     let filter = filter.to_lowercase();
///
///     value.to_lowercase().starts_with(&filter)
/// };
/// assert_eq!(false, filter("san", "New York",      0));
/// assert_eq!(false, filter("san", "Los Angeles",   1));
/// assert_eq!(false, filter("san", "Chicago",       2));
/// assert_eq!(false, filter("san", "Houston",       3));
/// assert_eq!(false, filter("san", "Phoenix",       4));
/// assert_eq!(false, filter("san", "Philadelphia",  5));
/// assert_eq!(true,  filter("san", "San Antonio",   6));
/// assert_eq!(true,  filter("san", "San Diego",     7));
/// assert_eq!(false, filter("san", "Dallas",        8));
/// assert_eq!(true,  filter("san", "San Francisco", 9));
/// assert_eq!(false, filter("san", "Austin",       10));
/// assert_eq!(false, filter("san", "Jacksonville", 11));
/// assert_eq!(true,  filter("san", "San Jose",     12));
/// ```
pub type Filter<'a, T> = &'a dyn Fn(&str, &T, &str, usize) -> bool;

/// Type alias to represent the function used to retrieve text input suggestions.
/// The function receives the current input and should return a collection of strings
/// containing the suggestions to be made to the user.
pub type Suggester<'a> = &'a dyn Fn(&str) -> Vec<String>;

/// Default page size when displaying options to the user.
pub const DEFAULT_PAGE_SIZE: usize = 7;

/// Default value of vim mode.
pub const DEFAULT_VIM_MODE: bool = false;
