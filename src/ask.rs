use std::error::Error;

use crate::config::PromptConfig;
use crate::confirm::Confirm;
use crate::confirm::ConfirmOptions;
use crate::input::Input;
use crate::input::InputOptions;
use crate::multiselect::MultiSelect;
use crate::multiselect::MultiSelectOptions;
use crate::question::Answer;
use crate::question::Prompt;
use crate::select::Select;
use crate::select::SelectOptions;

pub enum Question<'a> {
    MultiSelect(MultiSelectOptions<'a>),
    Select(SelectOptions<'a>),
    Input(InputOptions<'a>),
    Confirm(ConfirmOptions<'a>),
}

pub trait QuestionOptions<'a> {
    fn with_config(self, global_config: &'a PromptConfig) -> Self;
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

impl<'a> From<MultiSelectOptions<'a>> for Question<'a> {
    fn from(opt: MultiSelectOptions<'a>) -> Self {
        Self::MultiSelect(opt)
    }
}

impl<'a> From<SelectOptions<'a>> for Question<'a> {
    fn from(opt: SelectOptions<'a>) -> Self {
        Self::Select(opt)
    }
}

impl<'a> From<InputOptions<'a>> for Question<'a> {
    fn from(opt: InputOptions<'a>) -> Self {
        Self::Input(opt)
    }
}

impl<'a> From<ConfirmOptions<'a>> for Question<'a> {
    fn from(opt: ConfirmOptions<'a>) -> Self {
        Self::Confirm(opt)
    }
}
