//! Traits and structs used by prompts to validate user input before
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
//! This module also provides several macros as shorthands to the struct
//! constructor functions, exported with the `macros` feature.

use dyn_clone::DynClone;

use crate::{error::CustomUserError, list_option::ListOption};

/// Error message that is displayed to the users when their input is considered not
/// valid by registered validators.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum ErrorMessage {
    /// No custom message is defined, a standard one defined in the set
    /// [`RenderConfig`](crate::ui::RenderConfig) is used instead.
    #[default]
    Default,

    /// Custom error message, used instead of the standard one.
    Custom(String),
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
#[derive(Clone, Debug, PartialEq, Eq)]
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

/// Validator that receives a string slice as the input, such as [`Text`](crate::Text) and
/// [`Password`](crate::Password).
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
/// let validator = |input: &str| match input.chars().find(|c| c.is_numeric()) {
///     Some(_) => Ok(Validation::Valid),
///     None => Ok(Validation::Invalid(
///         "Your password should contain at least 1 digit".into(),
///     )),
/// };
///
/// assert_eq!(Validation::Valid, validator.validate("hunter2")?);
/// assert_eq!(
///     Validation::Invalid("Your password should contain at least 1 digit".into()),
///     validator.validate("password")?
/// );
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
pub trait StringValidator: DynClone {
    /// Confirm the given input string is a valid value.
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError>;
}

impl Clone for Box<dyn StringValidator> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl<F> StringValidator for F
where
    F: Fn(&str) -> Result<Validation, CustomUserError> + Clone,
{
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        (self)(input)
    }
}

/// Validator used in [`DateSelect`](crate::DateSelect) prompts.
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
/// let validator = |input: NaiveDate| {
///     if input.weekday() == Weekday::Sat || input.weekday() == Weekday::Sun {
///         Ok(Validation::Invalid("Weekends are not allowed".into()))
///     } else {
///         Ok(Validation::Valid)
///     }
/// };
///
/// assert_eq!(Validation::Valid, validator.validate(NaiveDate::from_ymd(2021, 7, 26))?);
/// assert_eq!(
///     Validation::Invalid("Weekends are not allowed".into()),
///     validator.validate(NaiveDate::from_ymd(2021, 7, 25))?
/// );
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[cfg(feature = "date")]
pub trait DateValidator: DynClone {
    /// Confirm the given input date is a valid value.
    fn validate(&self, input: chrono::NaiveDate) -> Result<Validation, CustomUserError>;
}

#[cfg(feature = "date")]
impl Clone for Box<dyn DateValidator> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

#[cfg(feature = "date")]
impl<F> DateValidator for F
where
    F: Fn(chrono::NaiveDate) -> Result<Validation, CustomUserError> + Clone,
{
    fn validate(&self, input: chrono::NaiveDate) -> Result<Validation, CustomUserError> {
        (self)(input)
    }
}

/// Validator used in [`MultiSelect`](crate::MultiSelect) prompts.
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
/// let validator = |input: &[ListOption<&&str>]| {
///     if input.len() <= 2 {
///         Ok(Validation::Valid)
///     } else {
///         Ok(Validation::Invalid("You should select at most two options".into()))
///     }
/// };
///
/// let mut ans = vec![ListOption::new(0, &"a"), ListOption::new(1, &"b")];
///
/// assert_eq!(Validation::Valid, validator.validate(&ans[..])?);
///
/// ans.push(ListOption::new(3, &"d"));
/// assert_eq!(
///     Validation::Invalid("You should select at most two options".into()),
///     validator.validate(&ans[..])?
/// );
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
pub trait MultiOptionValidator<T: ?Sized>: DynClone {
    /// Confirm the given input list is a valid value.
    fn validate(&self, input: &[ListOption<&T>]) -> Result<Validation, CustomUserError>;
}

impl<T> Clone for Box<dyn MultiOptionValidator<T>> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl<F, T> MultiOptionValidator<T> for F
where
    F: Fn(&[ListOption<&T>]) -> Result<Validation, CustomUserError> + Clone,
    T: ?Sized,
{
    fn validate(&self, input: &[ListOption<&T>]) -> Result<Validation, CustomUserError> {
        (self)(input)
    }
}

/// Validator used in [`CustomType`](crate::CustomType) prompts.
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
/// let validator = |input: &[ListOption<&&str>]| {
///     if input.len() <= 2 {
///         Ok(Validation::Valid)
///     } else {
///         Ok(Validation::Invalid("You should select at most two options".into()))
///     }
/// };
///
/// let mut ans = vec![ListOption::new(0, &"a"), ListOption::new(1, &"b")];
///
/// assert_eq!(Validation::Valid, validator.validate(&ans[..])?);
///
/// ans.push(ListOption::new(3, &"d"));
/// assert_eq!(
///     Validation::Invalid("You should select at most two options".into()),
///     validator.validate(&ans[..])?
/// );
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
pub trait CustomTypeValidator<T: ?Sized>: DynClone {
    /// Confirm the given input list is a valid value.
    fn validate(&self, input: &T) -> Result<Validation, CustomUserError>;
}

impl<T> Clone for Box<dyn CustomTypeValidator<T>> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl<F, T> CustomTypeValidator<T> for F
where
    F: Fn(&T) -> Result<Validation, CustomUserError> + Clone,
    T: ?Sized,
{
    fn validate(&self, input: &T) -> Result<Validation, CustomUserError> {
        (self)(input)
    }
}

/// Custom trait to call correct method to retrieve input length.
///
/// The method can vary depending on the type of input.
///
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
/// # Examples
///
/// ```
/// use inquire::validator::{StringValidator, Validation, ValueRequiredValidator};
///
/// let validator = ValueRequiredValidator::default();
/// assert_eq!(Validation::Valid, validator.validate("Generic input")?);
/// assert_eq!(Validation::Invalid("A response is required.".into()), validator.validate("")?);
///
/// let validator = ValueRequiredValidator::new("No empty!");
/// assert_eq!(Validation::Valid, validator.validate("Generic input")?);
/// assert_eq!(Validation::Invalid("No empty!".into()), validator.validate("")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[derive(Clone)]
pub struct ValueRequiredValidator {
    message: String,
}

impl ValueRequiredValidator {
    /// Create a new instance of this validator with given error message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for ValueRequiredValidator {
    /// Create a new instance of this validator with the default error message
    /// `A response is required`.
    fn default() -> Self {
        Self {
            message: "A response is required.".to_owned(),
        }
    }
}

impl StringValidator for ValueRequiredValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        Ok(if input.is_empty() {
            Validation::Invalid(self.message.as_str().into())
        } else {
            Validation::Valid
        })
    }
}

/// Shorthand for the built-in [`ValueRequiredValidator`] that checks whether the answer is not
/// empty.
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
/// let validator = required!();
/// assert_eq!(Validation::Valid, validator.validate("Generic input")?);
/// assert_eq!(Validation::Invalid("A response is required.".into()), validator.validate("")?);
///
/// let validator = required!("No empty!");
/// assert_eq!(Validation::Valid, validator.validate("Generic input")?);
/// assert_eq!(Validation::Invalid("No empty!".into()), validator.validate("")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! required {
    () => {
        $crate::validator::ValueRequiredValidator::default()
    };

    ($message:expr) => {
        $crate::validator::ValueRequiredValidator::new($message)
    };
}

/// Built-in validator that checks whether the answer length is smaller than
/// or equal to the specified threshold.
///
/// The validator uses a custom-built length function that
/// has a special implementation for strings which counts the number of
/// graphemes. See this [StackOverflow question](https://stackoverflow.com/questions/46290655/get-the-string-length-in-characters-in-rust).
///
/// # Examples
///
/// ```
/// use inquire::validator::{MaxLengthValidator, StringValidator, Validation};
///
/// let validator = MaxLengthValidator::new(5);
/// assert_eq!(Validation::Valid, validator.validate("Good")?);
/// assert_eq!(
///     Validation::Invalid("The length of the response should be at most 5".into()),
///     validator.validate("Terrible")?,
/// );
///
/// let validator = MaxLengthValidator::new(5).with_message("Not too large!");
/// assert_eq!(Validation::Valid, validator.validate("Good")?);
/// assert_eq!(Validation::Invalid("Not too large!".into()), validator.validate("Terrible")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[derive(Clone)]
pub struct MaxLengthValidator {
    limit: usize,
    message: String,
}

impl MaxLengthValidator {
    /// Create a new instance of this validator, requiring at most the given length, otherwise
    /// returning an error with default message.
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            message: format!("The length of the response should be at most {limit}"),
        }
    }

    /// Define a custom error message returned by the validator.
    /// Defaults to `The length of the response should be at most $length`.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    fn validate_inquire_length<T: InquireLength>(
        &self,
        input: T,
    ) -> Result<Validation, CustomUserError> {
        Ok(if input.inquire_length() <= self.limit {
            Validation::Valid
        } else {
            Validation::Invalid(self.message.as_str().into())
        })
    }
}

impl StringValidator for MaxLengthValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        self.validate_inquire_length(input)
    }
}

impl<T: ?Sized> MultiOptionValidator<T> for MaxLengthValidator {
    fn validate(&self, input: &[ListOption<&T>]) -> Result<Validation, CustomUserError> {
        self.validate_inquire_length(input)
    }
}

/// Shorthand for the built-in [`MaxLengthValidator`] that checks whether the answer length is
/// smaller than or equal to the specified threshold.
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
/// let validator = max_length!(5);
/// assert_eq!(Validation::Valid, validator.validate("Good")?);
/// assert_eq!(Validation::Invalid("The length of the response should be at most 5".into()), validator.validate("Terrible")?);
///
/// let validator = max_length!(5, "Not too large!");
/// assert_eq!(Validation::Valid, validator.validate("Good")?);
/// assert_eq!(Validation::Invalid("Not too large!".into()), validator.validate("Terrible")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! max_length {
    ($length:expr) => {
        $crate::validator::MaxLengthValidator::new($length)
    };

    ($length:expr, $message:expr) => {
        $crate::max_length!($length).with_message($message)
    };
}

/// Built-in validator that checks whether the answer length is larger than
/// or equal to the specified threshold.
///
/// The validator uses a custom-built length function that
/// has a special implementation for strings which counts the number of
/// graphemes. See this [StackOverflow question](https://stackoverflow.com/questions/46290655/get-the-string-length-in-characters-in-rust).
///
/// # Examples
///
/// ```
/// use inquire::validator::{MinLengthValidator, StringValidator, Validation};
///
/// let validator = MinLengthValidator::new(3);
/// assert_eq!(Validation::Valid, validator.validate("Yes")?);
/// assert_eq!(
///     Validation::Invalid("The length of the response should be at least 3".into()),
///     validator.validate("No")?,
/// );
///
/// let validator = MinLengthValidator::new(3).with_message("You have to give me more than that!");
/// assert_eq!(Validation::Valid, validator.validate("Yes")?);
/// assert_eq!(
///     Validation::Invalid("You have to give me more than that!".into()),
///     validator.validate("No")?,
/// );
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[derive(Clone)]
pub struct MinLengthValidator {
    limit: usize,
    message: String,
}

impl MinLengthValidator {
    /// Create a new instance of this validator, requiring at least the given length, otherwise
    /// returning an error with default message.
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            message: format!("The length of the response should be at least {limit}"),
        }
    }

    /// Define a custom error message returned by the validator.
    /// Defaults to `The length of the response should be at least $length`.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    fn validate_inquire_length<T: InquireLength>(
        &self,
        input: T,
    ) -> Result<Validation, CustomUserError> {
        Ok(if input.inquire_length() >= self.limit {
            Validation::Valid
        } else {
            Validation::Invalid(self.message.as_str().into())
        })
    }
}

impl StringValidator for MinLengthValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        self.validate_inquire_length(input)
    }
}

impl<T: ?Sized> MultiOptionValidator<T> for MinLengthValidator {
    fn validate(&self, input: &[ListOption<&T>]) -> Result<Validation, CustomUserError> {
        self.validate_inquire_length(input)
    }
}

/// Shorthand for the built-in [`MinLengthValidator`] that checks whether the answer length is
/// larger than or equal to the specified threshold.
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
/// let validator = min_length!(3);
/// assert_eq!(Validation::Valid, validator.validate("Yes")?);
/// assert_eq!(Validation::Invalid("The length of the response should be at least 3".into()), validator.validate("No")?);
///
/// let validator = min_length!(3, "You have to give me more than that!");
/// assert_eq!(Validation::Valid, validator.validate("Yes")?);
/// assert_eq!(Validation::Invalid("You have to give me more than that!".into()), validator.validate("No")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! min_length {
    ($length:expr) => {
        $crate::validator::MinLengthValidator::new($length)
    };

    ($length:expr, $message:expr) => {
        $crate::min_length!($length).with_message($message)
    };
}

/// Built-in validator that checks whether the answer length is equal to
/// the specified value.
///
/// The validator uses a custom-built length function that
/// has a special implementation for strings which counts the number of
/// graphemes. See this [StackOverflow question](https://stackoverflow.com/questions/46290655/get-the-string-length-in-characters-in-rust).
///
/// # Examples
///
/// ```
/// use inquire::validator::{ExactLengthValidator, StringValidator, Validation};
///
/// let validator = ExactLengthValidator::new(3);
/// assert_eq!(Validation::Valid, validator.validate("Yes")?);
/// assert_eq!(
///     Validation::Invalid("The length of the response should be 3".into()),
///     validator.validate("No")?,
/// );
///
/// let validator = ExactLengthValidator::new(3).with_message("Three characters please.");
/// assert_eq!(Validation::Valid, validator.validate("Yes")?);
/// assert_eq!(Validation::Invalid("Three characters please.".into()), validator.validate("No")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[derive(Clone)]
pub struct ExactLengthValidator {
    length: usize,
    message: String,
}

impl ExactLengthValidator {
    /// Create a new instance of this validator, requiring exactly the given length, otherwise
    /// returning an error with default message.
    pub fn new(length: usize) -> Self {
        Self {
            length,
            message: format!("The length of the response should be {length}"),
        }
    }

    /// Define a custom error message returned by the validator.
    /// Defaults to `The length of the response should be $length`.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    fn validate_inquire_length<T: InquireLength>(
        &self,
        input: T,
    ) -> Result<Validation, CustomUserError> {
        Ok(if input.inquire_length() == self.length {
            Validation::Valid
        } else {
            Validation::Invalid(self.message.as_str().into())
        })
    }
}

impl StringValidator for ExactLengthValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        self.validate_inquire_length(input)
    }
}

impl<T: ?Sized> MultiOptionValidator<T> for ExactLengthValidator {
    fn validate(&self, input: &[ListOption<&T>]) -> Result<Validation, CustomUserError> {
        self.validate_inquire_length(input)
    }
}

/// Shorthand for the built-in [`ExactLengthValidator`] that checks whether the answer length is
/// equal to the specified value.
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
/// let validator = length!(3);
/// assert_eq!(Validation::Valid, validator.validate("Yes")?);
/// assert_eq!(Validation::Invalid("The length of the response should be 3".into()), validator.validate("No")?);
///
/// let validator = length!(3, "Three characters please.");
/// assert_eq!(Validation::Valid, validator.validate("Yes")?);
/// assert_eq!(Validation::Invalid("Three characters please.".into()), validator.validate("No")?);
/// # Ok::<(), inquire::error::CustomUserError>(())
/// ```
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! length {
    ($length:expr) => {
        $crate::validator::ExactLengthValidator::new($length)
    };

    ($length:expr, $message:expr) => {
        $crate::length!($length).with_message($message)
    };
}

#[cfg(test)]
mod validators_test {
    use crate::{
        error::CustomUserError,
        list_option::ListOption,
        validator::{
            ExactLengthValidator, MaxLengthValidator, MinLengthValidator, MultiOptionValidator,
            StringValidator, Validation,
        },
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
        let validator = ExactLengthValidator::new(5);
        let validator: &dyn StringValidator = &validator;

        assert!(matches!(validator.validate("five!")?, Validation::Valid));
        assert!(matches!(validator.validate("♥️♥️♥️♥️♥️")?, Validation::Valid));
        assert!(matches!(
            validator.validate("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Valid
        ));

        assert!(matches!(
            validator.validate("five!!!")?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Invalid(_)
        ));

        Ok(())
    }

    #[test]
    fn slice_length() -> Result<(), CustomUserError> {
        let validator = ExactLengthValidator::new(5);
        let validator: &dyn MultiOptionValidator<str> = &validator;

        assert!(matches!(
            validator.validate(&build_option_vec(5))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(4))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(6))?,
            Validation::Invalid(_)
        ));

        Ok(())
    }

    #[test]
    fn string_max_length_counts_graphemes() -> Result<(), CustomUserError> {
        let validator = MaxLengthValidator::new(5);
        let validator: &dyn StringValidator = &validator;

        assert!(matches!(validator.validate("")?, Validation::Valid));
        assert!(matches!(validator.validate("five!")?, Validation::Valid));
        assert!(matches!(validator.validate("♥️♥️♥️♥️♥️")?, Validation::Valid));
        assert!(matches!(
            validator.validate("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Valid
        ));

        assert!(matches!(
            validator.validate("five!!!")?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate("♥️♥️♥️♥️♥️♥️")?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Invalid(_)
        ));

        Ok(())
    }

    #[test]
    fn slice_max_length() -> Result<(), CustomUserError> {
        let validator = MaxLengthValidator::new(5);
        let validator: &dyn MultiOptionValidator<str> = &validator;

        assert!(matches!(
            validator.validate(&build_option_vec(0))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(1))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(2))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(3))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(4))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(5))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(6))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(7))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(8))?,
            Validation::Invalid(_)
        ));

        Ok(())
    }

    #[test]
    fn string_min_length_counts_graphemes() -> Result<(), CustomUserError> {
        let validator = MinLengthValidator::new(5);
        let validator: &dyn StringValidator = &validator;

        assert!(matches!(validator.validate("")?, Validation::Invalid(_)));
        assert!(matches!(
            validator.validate("♥️♥️♥️♥️")?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate("mike")?,
            Validation::Invalid(_)
        ));

        assert!(matches!(validator.validate("five!")?, Validation::Valid));
        assert!(matches!(validator.validate("five!!!")?, Validation::Valid));
        assert!(matches!(validator.validate("♥️♥️♥️♥️♥️")?, Validation::Valid));
        assert!(matches!(validator.validate("♥️♥️♥️♥️♥️♥️")?, Validation::Valid));
        assert!(matches!(
            validator.validate("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate("🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️🤦🏼‍♂️")?,
            Validation::Valid
        ));

        Ok(())
    }

    #[test]
    fn slice_min_length() -> Result<(), CustomUserError> {
        let validator = MinLengthValidator::new(5);
        let validator: &dyn MultiOptionValidator<str> = &validator;

        assert!(matches!(
            validator.validate(&build_option_vec(0))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(1))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(2))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(3))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(4))?,
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(5))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(6))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(7))?,
            Validation::Valid
        ));
        assert!(matches!(
            validator.validate(&build_option_vec(8))?,
            Validation::Valid
        ));

        Ok(())
    }
}
