use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Answer {
    Confirm(bool),
    Content(String),
    Password(String),
    Option(OptionAnswer),
    MultipleOptions(Vec<OptionAnswer>),
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

    pub fn get_password(&self) -> &str {
        match self {
            Self::Password(val) => val,
            _ => panic!("Invalid answer variant"),
        }
    }

    pub fn into_password(self) -> String {
        match self {
            Self::Password(val) => val,
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
            Self::Password(_) => write!(f, "********"),
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

#[derive(Clone, Debug, PartialEq)]
pub struct OptionAnswer {
    pub index: usize,
    pub value: String,
}

impl OptionAnswer {
    pub(in crate) fn new(index: usize, value: &str) -> Self {
        Self {
            index,
            value: value.to_string(),
        }
    }

    #[allow(unused)]
    pub(in crate) fn from_str_list(vals: &[&str]) -> Vec<OptionAnswer> {
        vals.iter()
            .enumerate()
            .map(|(index, value)| Self {
                index,
                value: value.to_string(),
            })
            .collect()
    }

    #[allow(unused)]
    pub(in crate) fn from_idx_str_list(vals: &[(usize, &str)]) -> Vec<OptionAnswer> {
        vals.iter()
            .map(|(index, value)| Self {
                index: *index,
                value: value.to_string(),
            })
            .collect()
    }
}
