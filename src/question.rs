use std::error::Error;

use crate::{config::PromptConfig, survey::OptionAnswer, terminal::Terminal};

#[derive(Debug)]
pub enum Answer {
    Simple(String),
    Option(OptionAnswer),
    MultipleOptions(Vec<OptionAnswer>),
}

pub trait Question {
    fn render(&self, terminal: &Terminal);
    fn cleanup(&mut self, answer: &Answer) -> Result<(), Box<dyn Error>>;
    fn prompt(&mut self) -> Result<Answer, Box<dyn Error>>;
}
