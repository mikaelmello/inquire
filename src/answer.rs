use std::error::Error;
use std::fmt;

use crate::{survey::OptionAnswer, terminal::Terminal};

#[derive(Clone, Debug, PartialEq)]
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
    pub fn get_confirm(&self) -> bool {
        match self {
            Self::Confirm(val) => *val,
            _ => panic!("Invalid answer variant"),
        }
    }

    pub fn into_confirm(self) -> bool {
        match self {
            Self::Confirm(val) => val,
            _ => panic!("Invalid answer variant"),
        }
    }

    pub fn get_content(&self) -> &str {
        match self {
            Self::Content(val) => val,
            _ => panic!("Invalid answer variant"),
        }
    }

    pub fn into_content(self) -> String {
        match self {
            Self::Content(val) => val,
            _ => panic!("Invalid answer variant"),
        }
    }

    pub fn get_option(&self) -> &OptionAnswer {
        match self {
            Self::Option(val) => val,
            _ => panic!("Invalid answer variant"),
        }
    }

    pub fn into_option(self) -> OptionAnswer {
        match self {
            Self::Option(val) => val,
            _ => panic!("Invalid answer variant"),
        }
    }

    pub fn get_multiple_options(&self) -> &[OptionAnswer] {
        match self {
            Self::MultipleOptions(val) => val,
            _ => panic!("Invalid answer variant"),
        }
    }

    pub fn into_multiple_options(self) -> Vec<OptionAnswer> {
        match self {
            Self::MultipleOptions(val) => val,
            _ => panic!("Invalid answer variant"),
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
