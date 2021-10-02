//! Type aliases for functions used by prompts to validate user input before
//! returning the values to their callers.
//!
//! Validators receive the user input to a given prompt and decide whether
//! they are valid, returning `Ok(Validation::Valid)` in the process, or
//! invalid, returning `Ok(Validation::Invalid(ErrorMessage))`, where the
//! `ErrorMessage` content is an error message to be displayed to the end user.
//!
//! Validators can also return errors, which propagate to the caller prompt
//! and cause the prompt to return the error.
//!
//! When creating containers of validators, e.g. when calling `with_validators`
//! in a prompt, you might need to type hint the container with one of the types
//! below.
//!
//! This module also provides several built-in validators generated through macros,
//! exported with the `builtin_validators` feature.

use crate::{error::CustomUserError, list_option::ListOption};

/// Error message that is displayed to the users when their input is considered not
/// valid by registered validators.
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorMessage {
    /// No custom message is defined, a standard one defined in the set
    /// [`RenderConfig`](crate::RenderConfig) is used instead.
    Default,

    /// Custom error message, used instead of the standard one.
    Custom(String),
}

impl Default for ErrorMessage {
    fn default() -> Self {
        ErrorMessage::Default
    }
}

impl<T> From<T> for ErrorMessage
where
    T: ToString,
{
    fn from(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

/// The result type of validation operations when the execution of the validator
/// function succeeds.
#[derive(Clone, Debug, PartialEq)]
pub enum Validation {
    /// Variant that indicates that the input value is valid according to the validator.
    Valid,

    /// Variant that indicates that the input value is invalid according to the validator.
    ///
    /// The member represents a custom error message that will be displayed to the user when present.
    /// When empty a standard error message, configured via the RenderConfig struct, will be shown
    /// instead.
    Invalid(ErrorMessage),
}

/// Type alias for validators that receive a string slice as the input,
/// such as [Text](crate::Text) and [Password](crate::Password).
///
/// If the input provided by the user is valid, your validator should return `Ok(Validation::Valid)`.
///
/// If the input is not valid, your validator should return `Ok(Validation::Invalid(ErrorMessage))`,
/// where the content of `ErrorMessage` is recommended to be a string whose content will be displayed
/// to the user as an error message. It is also recommended that this value gives a helpful feedback to the user.
///
/// # Examples
///
/// ```
/// use inquire::validator::{StringValidator, Validation};
///
/// let validator: StringValidator = &|input| match input.chars().find(|c| c.is_numeric()) {
///     Some(_) => Ok(Validation::Valid),
///     None => Ok(Validation::Invalid(
///         "Your password should contain at least 1 digit".into(),
///     )),
/// };
///
/// assert_eq!(Validation::Valid, validator("hunter2")?);
/// assert_eq!(
///     Validation::Invalid("Your password should contain at least 1 digit".into()),
///     validator("password")?
/// );
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
pub type StringValidator<'a> = &'a dyn Fn(&str) -> Result<Validation, CustomUserError>;

/// Type alias for validators used in [`DateSelect`](crate::DateSelect) prompts.
///
/// If the input provided by the user is valid, your validator should return `Ok(Validation::Valid)`.
///
/// If the input is not valid, your validator should return `Ok(Validation::Invalid(ErrorMessage))`,
/// where the content of `ErrorMessage` is recommended to be a string whose content will be displayed
/// to the user as an error message. It is also recommended that this value gives a helpful feedback to the user.
///
/// # Examples
///
/// ```
/// use chrono::{Datelike, NaiveDate, Weekday};
/// use inquire::validator::{DateValidator, Validation};
///
/// let validator: DateValidator = &|input| {
///     if input.weekday() == Weekday::Sat || input.weekday() == Weekday::Sun {
///         Ok(Validation::Invalid("Weekends are not allowed".into()))
///     } else {
///         Ok(Validation::Valid)
///     }
/// };
///
/// assert_eq!(Validation::Valid, validator(NaiveDate::from_ymd(2021, 7, 26))?);
/// assert_eq!(
///     Validation::Invalid("Weekends are not allowed".into()),
///     validator(NaiveDate::from_ymd(2021, 7, 25))?
/// );
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[cfg(feature = "date")]
pub type DateValidator<'a> = &'a dyn Fn(chrono::NaiveDate) -> Result<Validation, CustomUserError>;

/// Type alias for validators used in [`MultiSelect`](crate::MultiSelect) prompts.
///
/// If the input provided by the user is valid, your validator should return `Ok(Validation::Valid)`.
///
/// If the input is not valid, your validator should return `Ok(Validation::Invalid(ErrorMessage))`,
/// where the content of `ErrorMessage` is recommended to be a string whose content will be displayed
/// to the user as an error message. It is also recommended that this value gives a helpful feedback to the user.
///
/// # Examples
///
/// ```
/// use inquire::list_option::ListOption;
/// use inquire::validator::{MultiOptionValidator, Validation};
///
/// let validator: MultiOptionValidator<&str> = &|input| {
///     if input.len() <= 2 {
///         Ok(Validation::Valid)
///     } else {
///         Ok(Validation::Invalid("You should select at most two options".into()))
///     }
/// };
///
/// let mut ans = vec![ListOption::new(0, &"a"), ListOption::new(1, &"b")];
///
/// assert_eq!(Validation::Valid, validator(&ans)?);
///
/// ans.push(ListOption::new(3, &"d"));
/// assert_eq!(
///     Validation::Invalid("You should select at most two options".into()),
///     validator(&ans)?
/// );
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
pub type MultiOptionValidator<'a, T> =
    &'a dyn Fn(&[ListOption<&T>]) -> Result<Validation, CustomUserError>;

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
/// use inquire::{required, validator::{StringValidator, Validation}};
///
/// let validator: StringValidator = required!();
/// assert_eq!(Validation::Valid, validator("Generic input")?);
/// assert_eq!(Validation::Invalid("A response is required.".into()), validator("")?);
///
/// let validator: StringValidator = required!("No empty!");
/// assert_eq!(Validation::Valid, validator("Generic input")?);
/// assert_eq!(Validation::Invalid("No empty!".into()), validator("")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! required {
    () => {
        $crate::required! {"A response is required."}
    };

    ($message:expr) => {{
        use $crate::validator::{ErrorMessage, Validation};
        &|a| match a.is_empty() {
            false => Ok(Validation::Valid),
            true => Ok(Validation::Invalid(String::from($message).into())),
        }
    }};
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
/// use inquire::{max_length, validator::{StringValidator, Validation}};
///
/// let validator: StringValidator = max_length!(5);
/// assert_eq!(Validation::Valid, validator("Good")?);
/// assert_eq!(Validation::Invalid("The length of the response should be at most 5".into()), validator("Terrible")?);
///
/// let validator: StringValidator = max_length!(5, "Not too large!");
/// assert_eq!(Validation::Valid, validator("Good")?);
/// assert_eq!(Validation::Invalid("Not too large!".into()), validator("Terrible")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! max_length {
    ($length:expr) => {
        $crate::max_length! {$length, format!("The length of the response should be at most {}", $length)}
    };

    ($length:expr, $message:expr) => {{
        use $crate::validator::{InquireLength, Validation};
        &|a| match a.inquire_length() {
            _len if _len <= $length => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid($message.into())),
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
/// use inquire::{min_length, validator::{StringValidator, Validation}};
///
/// let validator: StringValidator = min_length!(3);
/// assert_eq!(Validation::Valid, validator("Yes")?);
/// assert_eq!(Validation::Invalid("The length of the response should be at least 3".into()), validator("No")?);
///
/// let validator: StringValidator = min_length!(3, "You have to give me more than that!");
/// assert_eq!(Validation::Valid, validator("Yes")?);
/// assert_eq!(Validation::Invalid("You have to give me more than that!".into()), validator("No")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! min_length {
    ($length:expr) => {
        $crate::min_length! {$length, format!("The length of the response should be at least {}", $length)}
    };

    ($length:expr, $message:expr) => {{
        use $crate::validator::{InquireLength, Validation};
        &|a| match a.inquire_length() {
            _len if _len >= $length => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid($message.into())),
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
/// use inquire::{length, validator::{StringValidator, Validation}};
///
/// let validator: StringValidator = length!(3);
/// assert_eq!(Validation::Valid, validator("Yes")?);
/// assert_eq!(Validation::Invalid("The length of the response should be 3".into()), validator("No")?);
///
/// let validator: StringValidator = length!(3, "Three characters please.");
/// assert_eq!(Validation::Valid, validator("Yes")?);
/// assert_eq!(Validation::Invalid("Three characters please.".into()), validator("No")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! length {
    ($length:expr) => {
        $crate::length! {$length, format!("The length of the response should be {}", $length)}
    };

    ($length:expr, $message:expr) => {{
        use $crate::validator::{InquireLength, Validation};
        &|a| match a.inquire_length() {
            _len if _len == $length => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid($message.into())),
        }
    }};
}

#[cfg(test)]
#[cfg(feature = "builtin_validators")]
mod builtin_validators_test {
    use crate::{
        error::CustomUserError,
        list_option::ListOption,
        validator::{MultiOptionValidator, StringValidator, Validation},
    };

    fn build_option_vec(len: usize) -> Vec<ListOption<&'static str>> {
        let mut options = Vec::new();

        for i in 0..len {
            options.push(ListOption::new(i, ""));
        }

        options
    }

    #[test]
    fn string_length_counts_graphemes() -> Result<(), CustomUserError> {
        let validator: StringValidator = length!(5);

        assert!(matches!(validator("five!")?, Validation::Valid));
        assert!(matches!(validator("♥️♥️♥️♥️♥️")?, Validation::Valid));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Valid
        ));

        assert!(matches!(validator("five!!!")?, Validation::Invalid(_)));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Invalid(_)
        ));

        Ok(())
    }

    #[test]
    fn slice_length() -> Result<(), CustomUserError> {
        let validator: MultiOptionValidator<str> = length!(5);

        assert!(matches!(
            validator(&build_option_vec(5))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(4))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator(&build_option_vec(6))?,
            Validation::Invalid(_)
        ));

        Ok(())
    }

    #[test]
    fn string_max_length_counts_graphemes() -> Result<(), CustomUserError> {
        let validator: StringValidator = max_length!(5);

        assert!(matches!(validator("")?, Validation::Valid));
        assert!(matches!(validator("five!")?, Validation::Valid));
        assert!(matches!(validator("♥️♥️♥️♥️♥️")?, Validation::Valid));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Valid
        ));

        assert!(matches!(validator("five!!!")?, Validation::Invalid(_)));
        assert!(matches!(validator("♥️♥️♥️♥️♥️♥️")?, Validation::Invalid(_)));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Invalid(_)
        ));

        Ok(())
    }

    #[test]
    fn slice_max_length() -> Result<(), CustomUserError> {
        let validator: MultiOptionValidator<str> = max_length!(5);

        assert!(matches!(
            validator(&build_option_vec(0))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(1))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(2))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(3))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(4))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(5))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(6))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator(&build_option_vec(7))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator(&build_option_vec(8))?,
            Validation::Invalid(_)
        ));

        Ok(())
    }

    #[test]
    fn string_min_length_counts_graphemes() -> Result<(), CustomUserError> {
        let validator: StringValidator = min_length!(5);

        assert!(matches!(validator("")?, Validation::Invalid(_)));
        assert!(matches!(validator("♥️♥️♥️♥️")?, Validation::Invalid(_)));
        assert!(matches!(validator("mike")?, Validation::Invalid(_)));

        assert!(matches!(validator("five!")?, Validation::Valid));
        assert!(matches!(validator("five!!!")?, Validation::Valid));
        assert!(matches!(validator("♥️♥️♥️♥️♥️")?, Validation::Valid));
        assert!(matches!(validator("♥️♥️♥️♥️♥️♥️")?, Validation::Valid));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Valid
        ));
        assert!(matches!(
            validator("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Valid
        ));

        Ok(())
    }

    #[test]
    fn slice_min_length() -> Result<(), CustomUserError> {
        let validator: MultiOptionValidator<str> = min_length!(5);

        assert!(matches!(
            validator(&build_option_vec(0))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator(&build_option_vec(1))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator(&build_option_vec(2))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator(&build_option_vec(3))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator(&build_option_vec(4))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator(&build_option_vec(5))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(6))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(7))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator(&build_option_vec(8))?,
            Validation::Valid
        ));

        Ok(())
    }
}
