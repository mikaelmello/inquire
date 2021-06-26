use std::fmt;

/// Represents a selection made by the CLI user when prompted to select one or several
/// options among those presented.
#[derive(Clone, Debug, PartialEq)]
pub struct OptionAnswer {
    /// Index in the original collection of the selected option.
    pub index: usize,

    /// String value of the select option.
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

impl fmt::Display for OptionAnswer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
