#[derive(Debug, PartialEq)]
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
