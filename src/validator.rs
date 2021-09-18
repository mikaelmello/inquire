//! Type aliases for functions used by prompts to validate user input before
//! returning the values to their callers.
//!
//! Validators receive the user input to a given prompt and decide whether
//! they are valid, returning `Ok(())` in the process, or invalid, returning
//! `Err(String)`, where the `String` content is an error message to be
//! displayed to the end user.
//!
//! When creating containers of validators, e.g. when calling `with_validators`
//! in a prompt, you might need to type hint the container with one of the types
//! below.
//!
//! This module also provides several built-in validators generated through macros,
//! exported with the `builtin_validators` feature.
use crate::list_option::ListOption;

/// Type alias for validators that receive a string slice as the input,
/// such as [Text](crate::Text) and [Password](crate::Password).
///
/// If the input provided by the user is valid, your validator should return `Ok(())`.
///
/// If the input is not valid, your validator should return `Err(String)`,
/// where the content of `Err` is a string whose content will be displayed
/// to the user as an error message. It is recommended that this value gives
/// a helpful feedback to the user.
///
/// # Examples
///
/// ```
/// use inquire::validator::StringValidator;
///
/// let validator: StringValidator = &|input| match input.chars().find(|c| c.is_numeric()) {
///     Some(_) => Ok(()),
///     None => Err(String::from(
///         "Your password should contain at least 1 digit",
///     )),
/// };
///
/// assert_eq!(Ok(()), validator("hunter2"));
/// assert_eq!(
///     Err(String::from("Your password should contain at least 1 digit")),
///     validator("password")
/// );
/// ```
pub type StringValidator<'a> = &'a dyn Fn(&str) -> Result<(), String>;

/// Type alias for validators used in [`DateSelect`](crate::DateSelect) prompts.
///
/// If the input provided by the user is valid, your validator should return `Ok(())`.
///
/// If the input is not valid, your validator should return `Err(String)`,
/// where the content of `Err` is a string whose content will be displayed
/// to the user as an error message. It is recommended that this value gives
/// a helpful feedback to the user.
///
/// # Examples
///
/// ```
/// use chrono::{Datelike, NaiveDate, Weekday};
/// use inquire::validator::DateValidator;
///
/// let validator: DateValidator = &|input| {
///     if input.weekday() == Weekday::Sat || input.weekday() == Weekday::Sun {
///         Err(String::from("Weekends are not allowed"))
///     } else {
///         Ok(())
///     }
/// };
///
/// assert_eq!(Ok(()), validator(NaiveDate::from_ymd(2021, 7, 26)));
/// assert_eq!(
///     Err(String::from("Weekends are not allowed")),
///     validator(NaiveDate::from_ymd(2021, 7, 25))
/// );
/// ```
#[cfg(feature = "date")]
pub type DateValidator<'a> = &'a dyn Fn(chrono::NaiveDate) -> Result<(), String>;

/// Type alias for validators used in [`MultiSelect`](crate::MultiSelect) prompts.
///
/// If the input provided by the user is valid, your validator should return `Ok(())`.
///
/// If the input is not valid, your validator should return `Err(String)`,
/// where the content of `Err` is a string whose content will be displayed
/// to the user as an error message. It is recommended that this value gives
/// a helpful feedback to the user.
///
/// # Examples
///
/// ```
/// use inquire::list_option::ListOption;
/// use inquire::validator::MultiOptionValidator;
///
/// let validator = &|input: &[ListOption<&str>]| {
///     if input.len() <= 2 {
///         Ok(())
///     } else {
///         Err(String::from("You should select at most two options"))
///     }
/// };
///
/// let mut ans = vec![ListOption::new(0, "a"), ListOption::new(1, "b")];
/// assert_eq!(Ok(()), validator(&ans));
///
/// ans.push(ListOption::new(3, "d"));
/// assert_eq!(
///     Err(String::from("You should select at most two options")),
///     validator(&ans)
/// );
/// ```
pub type MultiOptionValidator<'a, T> = &'a dyn Fn(&[ListOption<&T>]) -> Result<(), String>;

/// Custom trait to call correct method to retrieve input length.
///
/// The method can vary depending on the type of input.

/// String inputs should count the number of graphemes, via
/// `.graphemes(true).count()`, instead of the number of bytes
/// via `.len()`. While simple slices should keep using `.len()`
pub trait InquireLength {
    /// String inputs should count the number of graphemes, via
    /// `.graphemes(true).count()`, instead of the number of bytes
    /// via `.len()`. While simple slices keep using `.len()`
    fn inquire_length(&self) -> usize;
}

impl InquireLength for &str {
    fn inquire_length(&self) -> usize {
        use unicode_segmentation::UnicodeSegmentation;

        self.graphemes(true).count()
    }
}

impl<T> InquireLength for &[T] {
    fn inquire_length(&self) -> usize {
        self.len()
    }
}

/// Built-in validator that checks whether the answer is not empty.
///
/// # Arguments
///
/// * `$message` - optional - Error message returned by the validator.
///   Defaults to "A response is required."
///
/// # Examples
///
/// ```
/// use inquire::{required, validator::{StringValidator}};
///
/// let validator: StringValidator = required!();
/// assert_eq!(Ok(()), validator("Generic input"));
/// assert_eq!(Err(String::from("A response is required.")), validator(""));
///
/// let validator: StringValidator = required!("No empty!");
/// assert_eq!(Ok(()), validator("Generic input"));
/// assert_eq!(Err(String::from("No empty!")), validator(""));
/// ```
#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! required {
    () => {
        $crate::required! {"A response is required."}
    };

    ($message:expr) => {
        &|a| match a.is_empty() {
            true => Err(String::from($message)),
            false => Ok(()),
        }
    };
}

/// Built-in validator that checks whether the answer length is smaller than
/// or equal to the specified threshold.
///
/// The validator uses a custom-built length function that
/// has a special implementation for strings which counts the number of
/// graphemes. See this [StackOverflow question](https://stackoverflow.com/questions/46290655/get-the-string-length-in-characters-in-rust).
///
/// # Arguments
///
/// * `$length` - Maximum length of the input.
/// * `$message` - optional - Error message returned by the validator.
///   Defaults to "The length of the response should be at most $length"
///
/// # Examples
///
/// ```
/// use inquire::{max_length, validator::{StringValidator}};
///
/// let validator: StringValidator = max_length!(5);
/// assert_eq!(Ok(()), validator("Good"));
/// assert_eq!(Err(String::from("The length of the response should be at most 5")), validator("Terrible"));
///
/// let validator: StringValidator = max_length!(5, "Not too large!");
/// assert_eq!(Ok(()), validator("Good"));
/// assert_eq!(Err(String::from("Not too large!")), validator("Terrible"));
/// ```
#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! max_length {
    ($length:expr) => {
        $crate::max_length! {$length, format!("The length of the response should be at most {}", $length)}
    };

    ($length:expr, $message:expr) => {{
        use $crate::validator::InquireLength;
        &|a| match a.inquire_length() {
            _len if _len <= $length => Ok(()),
            _ => Err(String::from($message)),
        }
    }};
}

/// Built-in validator that checks whether the answer length is larger than
/// or equal to the specified threshold.
///
/// The validator uses a custom-built length function that
/// has a special implementation for strings which counts the number of
/// graphemes. See this [StackOverflow question](https://stackoverflow.com/questions/46290655/get-the-string-length-in-characters-in-rust).
///
/// # Arguments
///
/// * `$length` - Minimum length of the input.
/// * `$message` - optional - Error message returned by the validator.
///   Defaults to "The length of the response should be at least $length"
///
/// # Examples
///
/// ```
/// use inquire::{min_length, validator::{StringValidator}};
///
/// let validator: StringValidator = min_length!(3);
/// assert_eq!(Ok(()), validator("Yes"));
/// assert_eq!(Err(String::from("The length of the response should be at least 3")), validator("No"));
///
/// let validator: StringValidator = min_length!(3, "You have to give me more than that!");
/// assert_eq!(Ok(()), validator("Yes"));
/// assert_eq!(Err(String::from("You have to give me more than that!")), validator("No"));
/// ```
#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! min_length {
    ($length:expr) => {
        $crate::min_length! {$length, format!("The length of the response should be at least {}", $length)}
    };

    ($length:expr, $message:expr) => {{
        use $crate::validator::InquireLength;
        &|a| match a.inquire_length() {
            _len if _len >= $length => Ok(()),
            _ => Err(String::from($message)),
        }
    }};
}

/// Built-in validator that checks whether the answer length is equal to
/// the specified value.
///
/// The validator uses a custom-built length function that
/// has a special implementation for strings which counts the number of
/// graphemes. See this [StackOverflow question](https://stackoverflow.com/questions/46290655/get-the-string-length-in-characters-in-rust).
///
/// # Arguments
///
/// * `$length` - Expected length of the input.
/// * `$message` - optional - Error message returned by the validator.
///   Defaults to "The length of the response should be $length"
///
/// # Examples
///
/// ```
/// use inquire::{length, validator::{StringValidator}};
///
/// let validator: StringValidator = length!(3);
/// assert_eq!(Ok(()), validator("Yes"));
/// assert_eq!(Err(String::from("The length of the response should be 3")), validator("No"));
///
/// let validator: StringValidator = length!(3, "Three characters please.");
/// assert_eq!(Ok(()), validator("Yes"));
/// assert_eq!(Err(String::from("Three characters please.")), validator("No"));
/// ```
#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! length {
    ($length:expr) => {
        $crate::length! {$length, format!("The length of the response should be {}", $length)}
    };

    ($length:expr, $message:expr) => {{
        use $crate::validator::InquireLength;
        &|a| match a.inquire_length() {
            _len if _len == $length => Ok(()),
            _ => Err(String::from($message)),
        }
    }};
}

#[cfg(test)]
#[cfg(feature = "builtin_validators")]
mod builtin_validators_test {
    use crate::{
        list_option::ListOption,
        validator::{MultiOptionValidator, StringValidator},
    };

    fn build_option_vec(len: usize) -> Vec<ListOption<&'static str>> {
        let mut options = Vec::new();

        for i in 0..len {
            options.push(ListOption::new(i, ""));
        }

        options
    }

    #[test]
    fn string_length_counts_graphemes() {
        let validator: StringValidator = length!(5);

        assert!(matches!(validator("five!"), Ok(_)));
        assert!(matches!(validator("♥️♥️♥️♥️♥️"), Ok(_)));
        assert!(matches!(validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️"), Ok(_)));

        assert!(matches!(validator("five!!!"), Err(_)));
        assert!(matches!(validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️"), Err(_)));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️"),
            Err(_)
        ));
    }

    #[test]
    fn slice_length() {
        let validator: MultiOptionValidator<str> = length!(5);

        assert!(matches!(validator(&build_option_vec(5)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(4)), Err(_)));
        assert!(matches!(validator(&build_option_vec(6)), Err(_)));
    }

    #[test]
    fn string_max_length_counts_graphemes() {
        let validator: StringValidator = max_length!(5);

        assert!(matches!(validator(""), Ok(_)));
        assert!(matches!(validator("five!"), Ok(_)));
        assert!(matches!(validator("♥️♥️♥️♥️♥️"), Ok(_)));
        assert!(matches!(validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️"), Ok(_)));

        assert!(matches!(validator("five!!!"), Err(_)));
        assert!(matches!(validator("♥️♥️♥️♥️♥️♥️"), Err(_)));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️"),
            Err(_)
        ));
    }

    #[test]
    fn slice_max_length() {
        let validator: MultiOptionValidator<str> = max_length!(5);

        assert!(matches!(validator(&build_option_vec(0)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(1)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(2)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(3)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(4)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(5)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(6)), Err(_)));
        assert!(matches!(validator(&build_option_vec(7)), Err(_)));
        assert!(matches!(validator(&build_option_vec(8)), Err(_)));
    }

    #[test]
    fn string_min_length_counts_graphemes() {
        let validator: StringValidator = min_length!(5);

        assert!(matches!(validator(""), Err(_)));
        assert!(matches!(validator("♥️♥️♥️♥️"), Err(_)));
        assert!(matches!(validator("mike"), Err(_)));

        assert!(matches!(validator("five!"), Ok(_)));
        assert!(matches!(validator("five!!!"), Ok(_)));
        assert!(matches!(validator("♥️♥️♥️♥️♥️"), Ok(_)));
        assert!(matches!(validator("♥️♥️♥️♥️♥️♥️"), Ok(_)));
        assert!(matches!(validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️"), Ok(_)));
        assert!(matches!(validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️"), Ok(_)));
    }

    #[test]
    fn slice_min_length() {
        let validator: MultiOptionValidator<str> = min_length!(5);

        assert!(matches!(validator(&build_option_vec(0)), Err(_)));
        assert!(matches!(validator(&build_option_vec(1)), Err(_)));
        assert!(matches!(validator(&build_option_vec(2)), Err(_)));
        assert!(matches!(validator(&build_option_vec(3)), Err(_)));
        assert!(matches!(validator(&build_option_vec(4)), Err(_)));
        assert!(matches!(validator(&build_option_vec(5)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(6)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(7)), Ok(_)));
        assert!(matches!(validator(&build_option_vec(8)), Ok(_)));
    }
}
