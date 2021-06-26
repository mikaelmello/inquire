use crate::OptionAnswer;

pub type StringValidator = fn(answer: &str) -> Result<(), &str>;

pub type MultiOptionValidator = fn(answer: &[OptionAnswer]) -> Result<(), &str>;
