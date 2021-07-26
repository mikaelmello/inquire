//! Utilities used to wrap user selections in [Select](crate::Select) and
//! [MultiSelect](crate::MultiSelect) prompts.

use std::fmt;

/// Represents a selection made by the user when prompted to select one or several
/// options among those presented.
///
/// It is essentially the return type of the [Select](crate::Select) and [MultiSelect](crate::MultiSelect)
/// prompts.
#[derive(Clone, Debug, PartialEq)]
pub struct OptionAnswer {
    /// Index of the selected option relative to the original (full) list passed to the prompt.
    pub index: usize,

    /// String value of the selected option.
    pub value: String,
}

impl OptionAnswer {
    /// Constructor for OptionAnswer.
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the option.
    /// * `value` - String value of the option
    ///
    /// # Examples
    ///
    /// ```
    /// use inquire::OptionAnswer;
    ///
    /// let answer = OptionAnswer::new(0, "a");
    /// ```
    pub fn new(index: usize, value: &str) -> Self {
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
