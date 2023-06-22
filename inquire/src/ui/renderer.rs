use std::{collections::BTreeSet, fmt::Display, io::Result};

use crate::{input::Input, list_option::ListOption, utils::Page, validator::ErrorMessage};

pub trait PromptRenderer {
    fn frame_setup(&mut self) -> Result<()>;
    fn frame_finish(&mut self) -> Result<()>;

    fn render_canceled_prompt(&mut self, prompt: &str) -> Result<()>;
    fn render_prompt_with_answer(&mut self, prompt: &str, answer: &str) -> Result<()>;

    fn render_error_message(&mut self, error: &ErrorMessage) -> Result<()>;
    fn render_help_message(&mut self, help: &str) -> Result<()>;
}

pub trait TextPromptRenderer: PromptRenderer {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> Result<()>;
    fn render_suggestions<D: Display>(&mut self, page: Page<'_, ListOption<D>>) -> Result<()>;
}

#[cfg(feature = "editor")]
pub trait EditorPromptRenderer: PromptRenderer {
    fn render_prompt(&mut self, prompt: &str, editor_command: &str) -> Result<()>;
}

pub trait SelectRenderer: PromptRenderer {
    fn render_select_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
    fn render_options<D: Display>(&mut self, page: Page<'_, ListOption<D>>) -> Result<()>;
}

pub trait MultiSelectRenderer: PromptRenderer {
    fn render_multiselect_prompt(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
    fn render_options<D: Display>(
        &mut self,
        page: Page<'_, ListOption<D>>,
        checked: &BTreeSet<usize>,
    ) -> Result<()>;
}

pub trait CustomTypePromptRenderer: PromptRenderer {
    fn render_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        cur_input: &Input,
    ) -> Result<()>;
}

pub trait PasswordPromptRenderer: PromptRenderer {
    fn render_prompt(&mut self, prompt: &str) -> Result<()>;
    fn render_prompt_with_masked_input(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
    fn render_prompt_with_full_input(&mut self, prompt: &str, cur_input: &Input) -> Result<()>;
}

#[cfg(feature = "date")]
pub trait DateSelectRenderer: PromptRenderer {
    fn render_calendar_prompt(&mut self, prompt: &str) -> Result<()>;

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
    ) -> Result<()>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}
