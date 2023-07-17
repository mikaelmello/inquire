use std::{collections::BTreeSet, fmt::Display};

use crate::{
    error::InquireResult, input::Input, list_option::ListOption, utils::Page,
    validator::ErrorMessage,
};

pub trait Renderer {
    fn frame_setup(&mut self) -> InquireResult<()>;
    fn frame_finish(&mut self) -> InquireResult<()>;

    fn render_canceled_prompt(&mut self, prompt: &str) -> InquireResult<()>;
    fn render_prompt_with_answer(&mut self, prompt: &str, answer: &str) -> InquireResult<()>;

    fn render_error_message(&mut self, error: &ErrorMessage) -> InquireResult<()>;
    fn render_help_message(&mut self, help: &str) -> InquireResult<()>;
}

pub trait TextPromptRenderer: Renderer {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> InquireResult<()>;
    fn render_suggestions<D: Display>(
        &mut self,
        page: Page<'_, ListOption<D>>,
    ) -> InquireResult<()>;
}

#[cfg(feature = "editor")]
pub trait EditorPromptRenderer: Renderer {
    fn render_prompt(&mut self, prompt: &str, editor_command: &str) -> InquireResult<()>;
}

pub trait SelectPromptRenderer: Renderer {
    fn render_select_prompt(&mut self, prompt: &str, cur_input: &Input) -> InquireResult<()>;
    fn render_options<D: Display>(&mut self, page: Page<'_, ListOption<D>>) -> InquireResult<()>;
}

pub trait MultiSelectPromptRenderer: Renderer {
    fn render_multiselect_prompt(&mut self, prompt: &str, cur_input: &Input) -> InquireResult<()>;
    fn render_options<D: Display>(
        &mut self,
        page: Page<'_, ListOption<D>>,
        checked: &BTreeSet<usize>,
    ) -> InquireResult<()>;
}

pub trait CustomTypePromptRenderer: Renderer {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> InquireResult<()>;
}

pub trait PasswordPromptRenderer: Renderer {
    fn render_prompt(&mut self, prompt: &str) -> InquireResult<()>;
    fn render_prompt_with_masked_input(
        &mut self,
        prompt: &str,
        cur_input: &Input,
    ) -> InquireResult<()>;
    fn render_prompt_with_full_input(
        &mut self,
        prompt: &str,
        cur_input: &Input,
    ) -> InquireResult<()>;
}

#[cfg(feature = "date")]
pub mod date {
    use crate::error::InquireResult;

    use super::Renderer;

    pub trait DateSelectPromptRenderer: Renderer {
        fn render_calendar_prompt(&mut self, prompt: &str) -> InquireResult<()>;

        #[allow(clippy::too_many_arguments)]
        fn render_calendar(
            &mut self,
            month: chrono::Month,
            year: i32,
            week_start: chrono::Weekday,
            today: chrono::NaiveDate,
            selected_date: chrono::NaiveDate,
            min_date: Option<chrono::NaiveDate>,
            max_date: Option<chrono::NaiveDate>,
        ) -> InquireResult<()>;
    }
}
