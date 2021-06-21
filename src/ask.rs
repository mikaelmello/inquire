use std::error::Error;

use crate::config::PromptConfig;
use crate::confirm::Confirm;
use crate::input::Input;
use crate::multiselect::MultiSelect;
use crate::question::Answer;
use crate::question::Prompt;
use crate::select::Select;
use crate::ConfirmOptions;
use crate::InputOptions;
use crate::MultiSelectOptions;
use crate::SelectOptions;

pub enum Question<'a> {
    MultiSelect(MultiSelectOptions<'a>),
    Select(SelectOptions<'a>),
    Input(InputOptions<'a>),
    Confirm(ConfirmOptions<'a>),
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
        match self {
            Question::MultiSelect(options) => MultiSelect::from(options).prompt(),
            Question::Select(options) => Select::from(options).prompt(),
            Question::Input(options) => Input::from(options).prompt(),
            Question::Confirm(options) => Confirm::from(options).prompt(),
        }
    }

    pub fn apply_global_config(
        questions: Vec<Self>,
        global_config: &'a PromptConfig<'a>,
    ) -> Vec<Self> {
        let with_global = |q| match q {
            Self::MultiSelect(opt) => Self::MultiSelect(opt.with_config(global_config)),
            Self::Select(opt) => Self::Select(opt.with_config(global_config)),
            Self::Input(opt) => Self::Input(opt.with_config(global_config)),
            Self::Confirm(opt) => Self::Confirm(opt.with_config(global_config)),
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
