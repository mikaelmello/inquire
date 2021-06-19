use std::error::Error;
use std::fmt;

use crate::{survey::OptionAnswer, terminal::Terminal};

#[derive(Debug)]
pub enum Answer {
    Confirm(bool),
    Content(String),
    Option(OptionAnswer),
    MultipleOptions(Vec<OptionAnswer>),
}

pub(in crate) trait Prompt {
    fn render(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error>;
    fn prompt(self) -> Result<Answer, Box<dyn Error>>;
}

impl Answer {
    pub fn get_confirm(&self) -> Option<bool> {
        match self {
            Self::Confirm(val) => Some(*val),
            _ => None,
        }
    }

    pub fn get_content(&self) -> Option<&str> {
        match self {
            Self::Content(val) => Some(val),
            _ => None,
        }
    }

    pub fn get_option(&self) -> Option<&OptionAnswer> {
        match self {
            Self::Option(val) => Some(val),
            _ => None,
        }
    }

    pub fn get_multiple_options(&self) -> Option<&[OptionAnswer]> {
        match self {
            Self::MultipleOptions(val) => Some(val),
            _ => None,
        }
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Confirm(val) => write!(
                f,
                "{}",
                match val {
                    true => "Yes",
                    false => "No",
                }
            ),
            Self::Content(val) => write!(f, "{}", val),
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
