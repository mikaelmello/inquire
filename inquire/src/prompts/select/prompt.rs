use std::{cmp::Reverse, fmt::Display};

use crate::{
    error::InquireResult,
    formatter::OptionFormatter,
    input::{Input, InputActionResult},
    list_option::ListOption,
    prompts::prompt::{ActionResult, Prompt},
    type_aliases::Scorer,
    ui::SelectBackend,
    utils::paginate,
    InquireError, Select,
};

use super::{action::SelectPromptAction, config::SelectConfig};

pub struct SelectPrompt<'a, T> {
    message: &'a str,
    config: SelectConfig,
    options: Vec<T>,
    string_options: Vec<String>,
    scored_options: Vec<usize>,
    help_message: Option<&'a str>,
    cursor_index: usize,
    input: Option<Input>,
    scorer: Scorer<'a, T>,
    formatter: OptionFormatter<'a, T>,
}

impl<'a, T> SelectPrompt<'a, T>
where
    T: Display,
{
    pub fn new(so: Select<'a, T>) -> InquireResult<Self> {
        if so.options.is_empty() {
            return Err(InquireError::InvalidConfiguration(
                "Available options can not be empty".into(),
            ));
        }

        if so.starting_cursor >= so.options.len() {
            return Err(InquireError::InvalidConfiguration(format!(
                "Starting cursor index {} is out-of-bounds for length {} of options",
                so.starting_cursor,
                &so.options.len()
            )));
        }

        let string_options = so.options.iter().map(T::to_string).collect();
        let scored_options = (0..so.options.len()).collect();

        let input = match so.filter_input_enabled {
            true => Some(Input::new_with(
                so.starting_filter_input.unwrap_or_default(),
            )),
            false => None,
        };

        Ok(Self {
            message: so.message,
            config: (&so).into(),
            options: so.options,
            string_options,
            scored_options,
            help_message: so.help_message,
            cursor_index: so.starting_cursor,
            input,
            scorer: so.scorer,
            formatter: so.formatter,
        })
    }

    fn move_cursor_up(&mut self, qty: usize, wrap: bool) -> ActionResult {
        let new_position = if wrap {
            let after_wrap = qty.saturating_sub(self.cursor_index);
            self.cursor_index
                .checked_sub(qty)
                .unwrap_or_else(|| self.scored_options.len().saturating_sub(after_wrap))
        } else {
            self.cursor_index.saturating_sub(qty)
        };

        self.update_cursor_position(new_position)
    }

    fn move_cursor_down(&mut self, qty: usize, wrap: bool) -> ActionResult {
        let mut new_position = self.cursor_index.saturating_add(qty);

        if new_position >= self.scored_options.len() {
            new_position = if self.scored_options.is_empty() {
                0
            } else if wrap {
                new_position % self.scored_options.len()
            } else {
                self.scored_options.len().saturating_sub(1)
            }
        }

        self.update_cursor_position(new_position)
    }

    fn update_cursor_position(&mut self, new_position: usize) -> ActionResult {
        if new_position != self.cursor_index {
            self.cursor_index = new_position;
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    fn has_answer_highlighted(&mut self) -> bool {
        self.scored_options.get(self.cursor_index).is_some()
    }

    fn get_final_answer(&mut self) -> ListOption<T> {
        // should only be called after current cursor index is validated
        // on has_answer_highlighted

        let index = *self.scored_options.get(self.cursor_index).unwrap();
        let value = self.options.swap_remove(index);

        ListOption::new(index, value)
    }

    fn run_scorer(&mut self) {
        let content = match &self.input {
            Some(input) => input.content(),
            None => return,
        };

        let mut options = self
            .options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| {
                (self.scorer)(content, opt, self.string_options.get(i).unwrap(), i)
                    .map(|score| (i, score))
            })
            .collect::<Vec<(usize, i64)>>();

        options.sort_unstable_by_key(|(_idx, score)| Reverse(*score));

        let new_scored_options = options.iter().map(|(idx, _)| *idx).collect::<Vec<usize>>();

        if self.scored_options == new_scored_options {
            return;
        }

        self.scored_options = new_scored_options;

        if self.config.reset_cursor {
            let _ = self.update_cursor_position(0);
        } else if self.scored_options.len() <= self.cursor_index {
            let _ = self.update_cursor_position(self.scored_options.len().saturating_sub(1));
        }
    }
}

impl<'a, Backend, T> Prompt<Backend> for SelectPrompt<'a, T>
where
    Backend: SelectBackend,
    T: Display,
{
    type Config = SelectConfig;
    type InnerAction = SelectPromptAction;
    type Output = ListOption<T>;

    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &SelectConfig {
        &self.config
    }

    fn format_answer(&self, answer: &ListOption<T>) -> String {
        (self.formatter)(answer.as_ref())
    }

    fn setup(&mut self) -> InquireResult<()> {
        self.run_scorer();
        Ok(())
    }

    fn submit(&mut self) -> InquireResult<Option<ListOption<T>>> {
        let answer = match self.has_answer_highlighted() {
            true => Some(self.get_final_answer()),
            false => None,
        };

        Ok(answer)
    }

    fn handle(&mut self, action: SelectPromptAction) -> InquireResult<ActionResult> {
        let result = match action {
            SelectPromptAction::MoveUp => self.move_cursor_up(1, true),
            SelectPromptAction::MoveDown => self.move_cursor_down(1, true),
            SelectPromptAction::PageUp => self.move_cursor_up(self.config.page_size, false),
            SelectPromptAction::PageDown => self.move_cursor_down(self.config.page_size, false),
            SelectPromptAction::MoveToStart => self.move_cursor_up(usize::MAX, false),
            SelectPromptAction::MoveToEnd => self.move_cursor_down(usize::MAX, false),

            SelectPromptAction::FilterInput(input_action) => match self.input.as_mut() {
                Some(input) => {
                    let result = input.handle(input_action);

                    if let InputActionResult::ContentChanged = result {
                        self.run_scorer();
                    }

                    result.into()
                }
                None => ActionResult::Clean,
            },
        };

        Ok(result)
    }

    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        let prompt = &self.message;

        backend.render_select_prompt(prompt, self.input.as_ref())?;

        let choices = self
            .scored_options
            .iter()
            .cloned()
            .map(|i| ListOption::new(i, self.options.get(i).unwrap()))
            .collect::<Vec<ListOption<&T>>>();

        let page = paginate(self.config.page_size, &choices, Some(self.cursor_index));

        backend.render_options(page)?;

        if let Some(help_message) = self.help_message {
            backend.render_help_message(help_message)?;
        }

        Ok(())
    }
}
