use crate::answer::OptionAnswer;

pub type StringFormatter = fn(answer: &str) -> &str;
pub type BoolFormatter<'a> = fn(answer: bool) -> &'a str;
pub type OptionFormatter = fn(answer: &OptionAnswer) -> String;
pub type MultiOptionFormatter = fn(answer: &[OptionAnswer]) -> String;

#[cfg(feature = "date")]
pub type DateFormatter = fn(answer: &chrono::NaiveDate) -> String;

pub(in crate) const DEFAULT_STRING_FORMATTER: StringFormatter = |val| val;

pub(in crate) const DEFAULT_BOOL_FORMATTER: BoolFormatter = |ans| match ans {
    true => "Yes",
    false => "No",
};

pub(in crate) const DEFAULT_OPTION_FORMATTER: OptionFormatter = |ans| ans.to_string();

pub(in crate) const DEFAULT_MULTI_OPTION_FORMATTER: MultiOptionFormatter = |ans| {
    ans.iter()
        .map(OptionAnswer::to_string)
        .collect::<Vec<String>>()
        .join(",")
};

#[cfg(feature = "date")]
pub(in crate) const DEFAULT_DATE_FORMATTER: DateFormatter =
    |val| val.format("%B %-e, %Y").to_string();
