//! This module contains the type aliases for functions called as formatters
//! of a given input.
//!
//! Formatters receive the user input to a given prompt and return a formatted
//! output `String`, which are displayed to the user after the submission
//! as their answer.
//!

use crate::answer::OptionAnswer;

/// Type alias for formatters that receive a string slice as the input,
/// such as [Text](crate::Text) and [Password](crate::Password).
///
/// Formatters receive the user input and return a [String] to be displayed
/// to the user as the final answer.
///
/// # Examples
///
/// ```
/// use inquire::formatter::StringFormatter;
///
/// let formatter: StringFormatter = &|i| i.to_lowercase();
/// assert_eq!(String::from("times square"), formatter("Times Square"));
/// assert_eq!(String::from("times square"), formatter("times square"));
/// ```
pub type StringFormatter<'a> = &'a dyn Fn(&str) -> String;

/// Type alias for formatters used in [Confirm](crate::Confirm) prompts.
///
/// Formatters receive the user input and return a [String] to be displayed
/// to the user as the final answer.
///
/// # Examples
///
/// ```
/// use inquire::formatter::BoolFormatter;
///
/// let formatter: BoolFormatter = &|i| match i {
///     true => String::from("si"),
///     false => String::from("no"),
/// };
/// assert_eq!(String::from("si"), formatter(true));
/// assert_eq!(String::from("no"), formatter(false));
/// ```
pub type BoolFormatter<'a> = &'a dyn Fn(bool) -> String;

/// Type alias for formatters used in [Select](crate::Select) prompts.
///
/// Formatters receive the user input and return a [String] to be displayed
/// to the user as the final answer.
///
/// # Examples
///
/// ```
/// use inquire::OptionAnswer;
/// use inquire::formatter::OptionFormatter;
///
/// let formatter: OptionFormatter = &|i| format!("Option {}: '{}'", i.index + 1, i.value);
/// assert_eq!(String::from("Option 1: 'a'"), formatter(&OptionAnswer::new(0, "a")));
/// assert_eq!(String::from("Option 2: 'b'"), formatter(&OptionAnswer::new(1, "b")));
/// ```
pub type OptionFormatter<'a> = &'a dyn Fn(&OptionAnswer) -> String;

/// Type alias for formatters used in [MultiSelect](crate::MultiSelect) prompts.
///
/// Formatters receive the user input and return a [String] to be displayed
/// to the user as the final answer.
///
/// # Examples
///
/// ```
/// use inquire::OptionAnswer;
/// use inquire::formatter::MultiOptionFormatter;
///
/// let formatter: MultiOptionFormatter = &|opts| {
///     let len = opts.len();
///     let options = match len {
///         1 => "option",
///         _ => "options",
///     };
///     format!("You selected {} {}", len, options)
/// };
///
/// let mut ans = vec![OptionAnswer::new(0, "a")];
/// assert_eq!(String::from("You selected 1 option"), formatter(&ans));
///
/// ans.push(OptionAnswer::new(3, "d"));
/// assert_eq!(String::from("You selected 2 options"), formatter(&ans));
/// ```
pub type MultiOptionFormatter<'a> = &'a dyn Fn(&[OptionAnswer]) -> String;

#[cfg(feature = "date")]

/// Type alias for formatters used in [DateSelect](crate::DateSelect) prompts.
///
/// Formatters receive the user input and return a [String] to be displayed
/// to the user as the final answer.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use inquire::formatter::DateFormatter;
///
/// let formatter: DateFormatter = &|val| val.format("%d/%m/%Y").to_string();
///
/// assert_eq!(
///     String::from("25/07/2021"),
///     formatter(NaiveDate::from_ymd(2021, 7, 25)),
/// );
/// ```
pub type DateFormatter<'a> = &'a dyn Fn(chrono::NaiveDate) -> String;

/// String formatter used by default in inputs that return a `String` as input.
/// Its behavior is to just echo the received input.
///
/// # Examples
///
/// ```
/// use inquire::formatter::DEFAULT_STRING_FORMATTER;
///
/// let formatter = DEFAULT_STRING_FORMATTER;
/// assert_eq!(String::from("Times Square"), formatter("Times Square"));
/// assert_eq!(String::from("times sQuare"), formatter("times sQuare"));
/// ```
pub const DEFAULT_STRING_FORMATTER: StringFormatter = &|val| String::from(val);

/// String formatter used by default in [Confirm](crate::Confirm) prompts.
/// Translates `bool` to `"Yes"` and `false` to `"No"`.
///
/// # Examples
///
/// ```
/// use inquire::formatter::DEFAULT_BOOL_FORMATTER;
///
/// let formatter = DEFAULT_BOOL_FORMATTER;
/// assert_eq!(String::from("Yes"), formatter(true));
/// assert_eq!(String::from("No"), formatter(false));
/// ```
pub const DEFAULT_BOOL_FORMATTER: BoolFormatter = &|ans| match ans {
    true => String::from("Yes"),
    false => String::from("No"),
};

/// String formatter used by default in [Select](crate::Select) prompts.
/// Simply prints the string value contained in the selected option.
///
/// # Examples
///
/// ```
/// use inquire::OptionAnswer;
/// use inquire::formatter::DEFAULT_OPTION_FORMATTER;
///
/// let formatter = DEFAULT_OPTION_FORMATTER;
/// assert_eq!(String::from("First option"), formatter(&OptionAnswer::new(0, "First option")));
/// assert_eq!(String::from("First option"), formatter(&OptionAnswer::new(11, "First option")));
/// ```
pub const DEFAULT_OPTION_FORMATTER: OptionFormatter = &|ans| ans.to_string();

/// String formatter used by default in [MultiSelect](crate::MultiSelect) prompts.
/// Prints the string value of all selected options, separated by commas.
///
/// # Examples
///
/// ```
/// use inquire::OptionAnswer;
/// use inquire::formatter::DEFAULT_MULTI_OPTION_FORMATTER;
///
/// let formatter = DEFAULT_MULTI_OPTION_FORMATTER;
///
/// let mut ans = vec![OptionAnswer::new(0, "New York")];
/// assert_eq!(String::from("New York"), formatter(&ans));
///
/// ans.push(OptionAnswer::new(3, "Seattle"));
/// assert_eq!(String::from("New York, Seattle"), formatter(&ans));
///
/// ans.push(OptionAnswer::new(7, "Vancouver"));
/// assert_eq!(String::from("New York, Seattle, Vancouver"), formatter(&ans));
/// ```
pub const DEFAULT_MULTI_OPTION_FORMATTER: MultiOptionFormatter = &|ans| {
    ans.iter()
        .map(OptionAnswer::to_string)
        .collect::<Vec<String>>()
        .join(", ")
};

#[cfg(feature = "date")]
/// String formatter used by default in [DateSelect](crate::DateSelect) prompts.
/// Prints the selected date in the format: Month Day, Year.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use inquire::formatter::DEFAULT_DATE_FORMATTER;
///
/// let formatter = DEFAULT_DATE_FORMATTER;
///
/// assert_eq!(
///     String::from("July 25, 2021"),
///     formatter(NaiveDate::from_ymd(2021, 7, 25)),
/// );
/// assert_eq!(
///     String::from("January 1, 2021"),
///     formatter(NaiveDate::from_ymd(2021, 1, 1)),
/// );
/// ```
pub const DEFAULT_DATE_FORMATTER: DateFormatter = &|val| val.format("%B %-e, %Y").to_string();
