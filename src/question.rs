use std::error::Error;
use std::fmt;

use crate::{survey::OptionAnswer, terminal::Terminal};

pub enum AskOptions {}

#[derive(Debug)]
pub enum Answer {
    Simple(String),
    Option(OptionAnswer),
    MultipleOptions(Vec<OptionAnswer>),
}

pub trait Question {
    fn render(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error>;
    fn cleanup(&mut self, answer: &Answer) -> Result<(), Box<dyn Error>>;
    fn prompt(&mut self) -> Result<Answer, Box<dyn Error>>;
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Simple(val) => write!(f, "{}", val),
            Self::Option(option) => write!(f, "{}", option.value),
            Self::MultipleOptions(options) => write!(
                f,
                "{}",
                options
                    .iter()
                    .map(|opt| opt.value.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            ),
        }
    }
}
