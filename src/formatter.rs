use crate::answer::OptionAnswer;

/// Type alias for formatters that receive a string slice as the input,
/// such as [Text](crate::Text) and [Password](crate::Password).
/// 
/// Formatters receive the user input and return a [String] to be displayed
/// to the user as the final formatting 
///
/// If the input provided by the user is invalid, your validator should return [Ok(())].
///
/// If the input is not valid, your validator should return [Err(String)],
/// where the content of [Err] is a string whose content will be displayed
/// to the user as an error message. It is recommended that this value gives
/// a helpful feedback to the user, e.g. "Your password should contain at least 8 characters".
pub type StringFormatter<'a> = &'a dyn Fn(&str) -> String;
pub type BoolFormatter<'a> = &'a dyn Fn(bool) -> String;
pub type OptionFormatter<'a> = &'a dyn Fn(&OptionAnswer) -> String;
pub type MultiOptionFormatter<'a> = &'a dyn Fn(&[OptionAnswer]) -> String;

#[cfg(feature = "date")]
pub type DateFormatter<'a> = &'a dyn Fn(chrono::NaiveDate) -> String;

pub(in crate) const DEFAULT_STRING_FORMATTER: StringFormatter = &|val| String::from(val);

pub(in crate) const DEFAULT_BOOL_FORMATTER: BoolFormatter = &|ans| match ans {
    true => String::from("Yes"),
    false => String::from("No"),
};

pub(in crate) const DEFAULT_OPTION_FORMATTER: OptionFormatter = &|ans| ans.to_string();

pub(in crate) const DEFAULT_MULTI_OPTION_FORMATTER: MultiOptionFormatter = &|ans| {
    ans.iter()
        .map(OptionAnswer::to_string)
        .collect::<Vec<String>>()
        .join(", ")
};

#[cfg(feature = "date")]
pub(in crate) const DEFAULT_DATE_FORMATTER: DateFormatter =
    &|val| val.format("%B %-e, %Y").to_string();
