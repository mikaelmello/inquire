pub type StringFormatter = fn(answer: &str) -> &str;
pub type BoolFormatter<'a> = fn(answer: bool) -> &'a str;

pub(in crate) const DEFAULT_STRING_FORMATTER: StringFormatter = |val| val;
pub(in crate) const DEFAULT_BOOL_FORMATTER: BoolFormatter = |ans| match ans {
    true => "Yes",
    false => "No",
};
