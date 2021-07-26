//! General type aliases and default values used by multiple prompts.

/// Type alias to represent the function used to filter options.
///
/// The function receives the current filter value and the value and index values of the option.
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
///
/// # Examples
///
/// ```
/// use inquire::config::DEFAULT_FILTER;
///
/// let filter = DEFAULT_FILTER;
/// assert_eq!(false, filter("sa", "New York",      0));
/// assert_eq!(true,  filter("sa", "Sacramento",     1));
/// assert_eq!(true,  filter("sa", "Kansas",         2));
/// assert_eq!(true,  filter("sa", "Mesa",           3));
/// assert_eq!(false, filter("sa", "Phoenix",       4));
/// assert_eq!(false, filter("sa", "Philadelphia",  5));
/// assert_eq!(true,  filter("sa", "San Antonio",   6));
/// assert_eq!(true,  filter("sa", "San Diego",     7));
/// assert_eq!(false, filter("sa", "Dallas",        8));
/// assert_eq!(true,  filter("sa", "San Francisco", 9));
/// assert_eq!(false, filter("sa", "Austin",       10));
/// assert_eq!(false, filter("sa", "Jacksonville", 11));
/// assert_eq!(true,  filter("sa", "San Jose",     12));
/// ```
pub const DEFAULT_FILTER: Filter = &|filter, value, _| -> bool {
    let filter = filter.to_lowercase();

    value.to_lowercase().contains(&filter)
};
