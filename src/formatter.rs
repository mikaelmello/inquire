pub type StringFormatter = fn(answer: &str) -> &str;
pub type BoolFormatter<'a> = fn(answer: bool) -> &'a str;
