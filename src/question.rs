use std::error::Error;

use crate::{config::PromptConfig, survey::OptionAnswer};

pub enum Answer {
    Simple(String),
    Option(OptionAnswer),
    MultipleOptions(Vec<OptionAnswer>),
}

pub trait Question {
    fn render(&self);
    fn cleanup(&mut self);
    fn prompt(&mut self, config: &PromptConfig) -> Result<Answer, Box<dyn Error>>;
}
