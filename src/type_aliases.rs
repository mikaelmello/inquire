//! General type aliases.

use crate::error::CustomUserError;

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
/// use inquire::type_aliases::Filter;
///
/// let filter: Filter<str> = &|filter, _, string_value, _| -> bool {
///     let filter = filter.to_lowercase();
///
///     string_value.to_lowercase().starts_with(&filter)
/// };
/// assert_eq!(false, filter("san", "New York",      "New York",      0));
/// assert_eq!(false, filter("san", "Los Angeles",   "Los Angeles",   1));
/// assert_eq!(false, filter("san", "Chicago",       "Chicago",       2));
/// assert_eq!(false, filter("san", "Houston",       "Houston",       3));
/// assert_eq!(false, filter("san", "Phoenix",       "Phoenix",       4));
/// assert_eq!(false, filter("san", "Philadelphia",  "Philadelphia",  5));
/// assert_eq!(true,  filter("san", "San Antonio",   "San Antonio",   6));
/// assert_eq!(true,  filter("san", "San Diego",     "San Diego",     7));
/// assert_eq!(false, filter("san", "Dallas",        "Dallas",        8));
/// assert_eq!(true,  filter("san", "San Francisco", "San Francisco", 9));
/// assert_eq!(false, filter("san", "Austin",        "Austin",       10));
/// assert_eq!(false, filter("san", "Jacksonville",  "Jacksonville", 11));
/// assert_eq!(true,  filter("san", "San Jose",      "San Jose",     12));
/// ```
pub type Filter<'a, T> = &'a dyn Fn(&str, &T, &str, usize) -> bool;

/// Type alias to represent the function used to retrieve text input suggestions.
/// The function receives the current input and should return a collection of strings
/// containing the suggestions to be made to the user.
pub type Suggester<'a> = &'a dyn Fn(&str) -> Result<Vec<String>, CustomUserError>;

/// Type alias to represent the function used to retrieve an optional autocompletion suggestion.
/// The function receives the current input and should return the suggestion (if any)
/// that will replace the current input.
pub type Completer<'a> = &'a dyn Fn(&str) -> Result<Option<String>, CustomUserError>;
