use std::error::Error;

use crate::answer::Answer;

pub type Filter = fn(filter: &str, value: &str, index: usize) -> bool;
pub type Transformer = fn(answer: &Answer) -> String;
pub type Validator = fn(answer: &Answer) -> Result<(), Box<dyn Error>>;

#[derive(Copy, Clone, Default)]
pub struct PromptConfig<'a> {
    pub page_size: Option<usize>,
    pub filter: Option<Filter>,
    pub confirm_default: Option<bool>,
    pub transformer: Option<Transformer>,
    pub validator: Option<Validator>,
    pub help_message: Option<&'a str>,
    pub keep_filter: Option<bool>,
    pub vim_mode: Option<bool>,
}

pub(in crate) const DEFAULT_PAGE_SIZE: usize = 7;
pub(in crate) const DEFAULT_VIM_MODE: bool = false;
pub(in crate) const DEFAULT_KEEP_FILTER: bool = true;

pub(in crate) const DEFAULT_FILTER: Filter = |filter: &str, value: &str, _| -> bool {
    let filter = filter.to_lowercase();

    value.to_lowercase().contains(&filter)
};

pub(in crate) const DEFAULT_TRANSFORMER: Transformer =
    |answer: &Answer| -> String { answer.to_string() };

pub(in crate) const DEFAULT_VALIDATOR: Validator = |answer: &Answer| -> Result<(), Box<dyn Error>> {
    match answer {
        Answer::Confirm(_) => Ok(()),
        Answer::Content(_) => Ok(()),
        Answer::Option(_) => Ok(()),
        Answer::MultipleOptions(_) => Ok(()),
        Answer::Password(_) => Ok(()),
    }
};
