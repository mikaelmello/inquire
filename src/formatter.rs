use crate::answer::OptionAnswer;

pub type StringFormatter = fn(answer: &str) -> &str;
pub type BoolFormatter<'a> = fn(answer: bool) -> &'a str;
pub type OptionFormatter = fn(answer: &OptionAnswer) -> String;
pub type MultiOptionFormatter = fn(answer: &[OptionAnswer]) -> String;

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
