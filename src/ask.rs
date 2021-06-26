use std::error::Error;

use crate::answer::Answer;
use crate::config::PromptConfig;
use crate::multiselect::MultiSelect;
use crate::renderer::Renderer;
use crate::select::Select;
use crate::terminal::Terminal;
use crate::MultiSelectOptions;
use crate::Prompt;
use crate::SelectOptions;

pub enum Question<'a> {
    MultiSelect(MultiSelectOptions<'a>),
    Select(SelectOptions<'a>),
}

pub trait QuestionOptions<'a> {
    fn with_config(self, global_config: &'a PromptConfig) -> Self;
    fn into_question(self) -> Question<'a>;
}

pub trait AskMany {
    fn ask(self) -> Result<Vec<Answer>, Box<dyn Error>>;
}

impl<'a> Question<'a> {
    pub fn ask(self) -> Result<Answer, Box<dyn Error>> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;

        let answer = match self {
            Question::MultiSelect(options) => MultiSelect::from(options).prompt(&mut renderer),
            Question::Select(options) => Select::from(options).prompt(&mut renderer),
        }?;

        Ok(answer)
    }

    pub fn apply_global_config(
        questions: Vec<Self>,
        global_config: &'a PromptConfig<'a>,
    ) -> Vec<Self> {
        let with_global = |q| match q {
            Self::MultiSelect(opt) => Self::MultiSelect(opt.with_config(global_config)),
            Self::Select(opt) => Self::Select(opt.with_config(global_config)),
        };

        questions.into_iter().map(with_global).collect()
    }
}

impl<'a, I> AskMany for I
where
    I: Iterator<Item = Question<'a>>,
{
    fn ask(self) -> Result<Vec<Answer>, Box<dyn Error>> {
        self.map(Question::ask).collect()
    }
}
