//! General type aliases.

use crate::error::CustomUserError;

/// Type alias to represent the function used to Score and filter options.
///
/// The function receives:
/// - Current user input, filter value
/// - Current option being evaluated, with type preserved
/// - String value of the current option
/// - Index of the current option in the original list
///
/// The return type should be a score determining the order options should be displayed to the user.
/// The greater the score, the higher on the list it will be displayed.
///
/// # Examples
///
/// ```
/// use inquire::type_aliases::Scorer;
///
/// // Implement simpler 'contains' filter that maintains the current order
/// // If all scores are the same, no sorting will occur
/// let filter: Scorer<str> =
///     &|input, _option, string_value, _idx| -> Option<i64> {
///        let filter = input.to_lowercase();
///        match string_value.to_lowercase().contains(&filter) {
///            true => Some(0),
///            false => None,
///        }
///     };
///
/// assert_eq!(None, filter("sa", "New York",      "New York",      0));
/// assert_eq!(None, filter("sa", "Los Angeles",   "Los Angeles",   1));
/// assert_eq!(None, filter("sa", "Chicago",       "Chicago",       2));
/// assert_eq!(None, filter("sa", "Houston",       "Houston",       3));
/// assert_eq!(None, filter("sa", "Phoenix",       "Phoenix",       4));
/// assert_eq!(None, filter("sa", "Philadelphia",  "Philadelphia",  5));
/// assert_eq!(Some(0), filter("sa", "San Antonio",   "San Antonio",   6));
/// assert_eq!(Some(0), filter("sa", "San Diego",     "San Diego",     7));
/// assert_eq!(None, filter("sa", "Dallas",        "Dallas",        8));
/// assert_eq!(Some(0), filter("sa", "San Francisco", "San Francisco", 9));
/// assert_eq!(None, filter("sa", "Austin",        "Austin",       10));
/// assert_eq!(None, filter("sa", "Jacksonville",  "Jacksonville", 11));
/// assert_eq!(Some(0), filter("sa", "San Jose",      "San Jose",     12));
///```
///
///
///
/// Default implementation for fuzzy search (almost)
///```
/// use inquire::type_aliases::Scorer;
/// use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
///
/// pub const DEFAULT_SCORER: Scorer<str> =
///     &|input, _option, string_value, _idx| -> Option<i64> {
///         let matcher = SkimMatcherV2::default().ignore_case();
///         matcher.fuzzy_match(string_value, input)
///     };
///
/// assert_eq!(None,     DEFAULT_SCORER("sa", &"New York",      "New York",      0));
/// assert_eq!(Some(49), DEFAULT_SCORER("sa", &"Sacramento",    "Sacramento",    1));
/// assert_eq!(Some(35), DEFAULT_SCORER("sa", &"Kansas",        "Kansas",        2));
/// assert_eq!(Some(35), DEFAULT_SCORER("sa", &"Mesa",          "Mesa",          3));
/// assert_eq!(None,     DEFAULT_SCORER("sa", &"Phoenix",       "Phoenix",       4));
/// assert_eq!(None,     DEFAULT_SCORER("sa", &"Philadelphia",  "Philadelphia",  5));
/// assert_eq!(Some(49), DEFAULT_SCORER("sa", &"San Antonio",   "San Antonio",   6));
/// assert_eq!(Some(49), DEFAULT_SCORER("sa", &"San Diego",     "San Diego",     7));
/// assert_eq!(None,     DEFAULT_SCORER("sa", &"Dallas",        "Dallas",        8));
/// assert_eq!(Some(49), DEFAULT_SCORER("sa", &"San Francisco", "San Francisco", 9));
/// assert_eq!(None,     DEFAULT_SCORER("sa", &"Austin",        "Austin",        10));
/// assert_eq!(None,     DEFAULT_SCORER("sa", &"Jacksonville",  "Jacksonville",  11));
/// assert_eq!(Some(49), DEFAULT_SCORER("sa", &"San Jose",      "San Jose",      12));
/// ```
pub type Scorer<'a, T> = &'a dyn Fn(&str, &T, &str, usize) -> Option<i64>;

/// Type alias to represent the function used to retrieve text input suggestions.
/// The function receives the current input and should return a collection of strings
/// containing the suggestions to be made to the user.
pub type Suggester<'a> = &'a dyn Fn(&str) -> Result<Vec<String>, CustomUserError>;

/// Type alias to represent the function used to retrieve an optional autocompletion suggestion.
/// The function receives the current input and should return the suggestion (if any)
/// that will replace the current input.
pub type Completer<'a> = &'a dyn Fn(&str) -> Result<Option<String>, CustomUserError>;
