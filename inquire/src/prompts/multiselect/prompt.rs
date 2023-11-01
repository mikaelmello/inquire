use std::{cmp::Reverse, collections::BTreeSet, fmt::Display};

use crate::{
    error::InquireResult,
    formatter::MultiOptionFormatter,
    input::{Input, InputActionResult},
    list_option::ListOption,
    prompts::prompt::{ActionResult, Prompt},
    type_aliases::Scorer,
    ui::MultiSelectBackend,
    utils::paginate,
    validator::{ErrorMessage, MultiOptionValidator, Validation},
    InquireError, MultiSelect,
};

use super::{action::MultiSelectPromptAction, config::MultiSelectConfig};

pub struct MultiSelectPrompt<'a, T> {
    message: &'a str,
    config: MultiSelectConfig,
    options: Vec<T>,
    string_options: Vec<String>,
    help_message: Option<&'a str>,
    cursor_index: usize,
    checked: BTreeSet<usize>,
    input: Input,
    scored_options: Vec<usize>,
    scorer: Scorer<'a, T>,
    formatter: MultiOptionFormatter<'a, T>,
    validator: Option<Box<dyn MultiOptionValidator<T>>>,
    error: Option<ErrorMessage>,
}

impl<'a, T> MultiSelectPrompt<'a, T>
where
    T: Display,
{
    pub fn new(mso: MultiSelect<'a, T>) -> InquireResult<Self> {
        if mso.options.is_empty() {
            return Err(InquireError::InvalidConfiguration(
                "Available options can not be empty".into(),
            ));
        }
        if let Some(default) = &mso.default {
            for i in default {
                if i >= &mso.options.len() {
                    return Err(InquireError::InvalidConfiguration(format!(
                        "Index {} is out-of-bounds for length {} of options",
                        i,
                        &mso.options.len()
                    )));
                }
            }
        }

        let string_options = mso.options.iter().map(T::to_string).collect();
        let scored_options = (0..mso.options.len()).collect();
        let checked_options = mso
            .default
            .as_ref()
            .map(|d| {
                d.iter()
                    .cloned()
                    .filter(|i| *i < mso.options.len())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            message: mso.message,
            config: (&mso).into(),
            options: mso.options,
            string_options,
            scored_options,
            help_message: mso.help_message,
            cursor_index: mso.starting_cursor,
            input: mso
                .starting_filter_input
                .map(Input::new_with)
                .unwrap_or_else(Input::new),
            scorer: mso.scorer,
            formatter: mso.formatter,
            validator: mso.validator,
            error: None,
            checked: checked_options,
        })
    }

    fn score_options(&self) -> Vec<(usize, i64)> {
        self.options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| {
                (self.scorer)(
                    self.input.content(),
                    opt,
                    self.string_options.get(i).unwrap(),
                    i,
                )
                .map(|score| (i, score))
            })
            .collect::<Vec<(usize, i64)>>()
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

    fn toggle_cursor_selection(&mut self) -> ActionResult {
        let idx = match self.scored_options.get(self.cursor_index) {
            Some(val) => val,
            None => return ActionResult::Clean,
        };

        if self.checked.contains(idx) {
            self.checked.remove(idx);
        } else {
            self.checked.insert(*idx);
        }

        if !self.config.keep_filter {
            self.input.clear();
        }

        ActionResult::NeedsRedraw
    }

    fn validate_current_answer(&self) -> InquireResult<Validation> {
        if let Some(validator) = &self.validator {
            let selected_options = self
                .options
                .iter()
                .enumerate()
                .filter_map(|(idx, opt)| match &self.checked.contains(&idx) {
                    true => Some(ListOption::new(idx, opt)),
                    false => None,
                })
                .collect::<Vec<_>>();

            let res = validator.validate(&selected_options)?;
            Ok(res)
        } else {
            Ok(Validation::Valid)
        }
    }

    fn get_final_answer(&mut self) -> Vec<ListOption<T>> {
        let mut answer = vec![];

        // by iterating in descending order, we can safely
        // swap remove because the elements to the right
        // that we did not remove will not matter anymore.
        for index in self.checked.iter().rev() {
            let index = *index;
            let value = self.options.swap_remove(index);
            let lo = ListOption::new(index, value);
            answer.push(lo);
        }
        answer.reverse();

        answer
    }

    fn run_scorer(&mut self) {
        let mut options = self.score_options();
        options.sort_unstable_by_key(|(_idx, score)| Reverse(*score));

        self.scored_options = options.into_iter().map(|(idx, _)| idx).collect();
        if self.config.reset_cursor {
            let _ = self.update_cursor_position(0);
        } else if self.scored_options.len() <= self.cursor_index {
            let _ = self.update_cursor_position(self.scored_options.len().saturating_sub(1));
        }
    }
}

impl<'a, Backend, T> Prompt<Backend> for MultiSelectPrompt<'a, T>
where
    Backend: MultiSelectBackend,
    T: Display,
{
    type Config = MultiSelectConfig;
    type InnerAction = MultiSelectPromptAction;
    type Output = Vec<ListOption<T>>;

    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &MultiSelectConfig {
        &self.config
    }

    fn format_answer(&self, answer: &Vec<ListOption<T>>) -> String {
        let refs: Vec<ListOption<&T>> = answer.iter().map(ListOption::as_ref).collect();
        (self.formatter)(&refs)
    }

    fn setup(&mut self) -> InquireResult<()> {
        self.run_scorer();
        Ok(())
    }

    fn submit(&mut self) -> InquireResult<Option<Vec<ListOption<T>>>> {
        let answer = match self.validate_current_answer()? {
            Validation::Valid => Some(self.get_final_answer()),
            Validation::Invalid(msg) => {
                self.error = Some(msg);
                None
            }
        };

        Ok(answer)
    }

    fn handle(&mut self, action: MultiSelectPromptAction) -> InquireResult<ActionResult> {
        let result = match action {
            MultiSelectPromptAction::MoveUp => self.move_cursor_up(1, true),
            MultiSelectPromptAction::MoveDown => self.move_cursor_down(1, true),
            MultiSelectPromptAction::PageUp => self.move_cursor_up(self.config.page_size, false),
            MultiSelectPromptAction::PageDown => {
                self.move_cursor_down(self.config.page_size, false)
            }
            MultiSelectPromptAction::MoveToStart => self.move_cursor_up(usize::MAX, false),
            MultiSelectPromptAction::MoveToEnd => self.move_cursor_down(usize::MAX, false),
            MultiSelectPromptAction::ToggleCurrentOption => self.toggle_cursor_selection(),
            MultiSelectPromptAction::SelectAll => {
                self.checked.clear();
                for idx in &self.scored_options {
                    self.checked.insert(*idx);
                }

                if !self.config.keep_filter {
                    self.input.clear();
                }

                ActionResult::NeedsRedraw
            }
            MultiSelectPromptAction::ClearSelections => {
                self.checked.clear();

                if !self.config.keep_filter {
                    self.input.clear();
                }

                ActionResult::NeedsRedraw
            }
            MultiSelectPromptAction::FilterInput(input_action) => {
                let result = self.input.handle(input_action);

                if let InputActionResult::ContentChanged = result {
                    self.run_scorer();
                }

                result.into()
            }
        };

        Ok(result)
    }

    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        let prompt = &self.message;

        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        backend.render_multiselect_prompt(prompt, &self.input)?;

        let choices = self
            .scored_options
            .iter()
            .cloned()
            .map(|i| ListOption::new(i, self.options.get(i).unwrap()))
            .collect::<Vec<ListOption<&T>>>();

        let page = paginate(self.config.page_size, &choices, Some(self.cursor_index));

        backend.render_options(page, &self.checked)?;

        if let Some(help_message) = self.help_message {
            backend.render_help_message(help_message)?;
        }

        Ok(())
    }
}
