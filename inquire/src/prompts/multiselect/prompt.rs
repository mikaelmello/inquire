use std::{collections::BTreeSet, fmt::Display};

use crate::{
    error::InquireResult,
    formatter::MultiOptionFormatter,
    input::Input,
    list_option::ListOption,
    prompt::{HandleResult, Prompt},
    type_aliases::Filter,
    ui::MultiSelectBackend,
    utils::paginate,
    validator::{ErrorMessage, MultiOptionValidator, Validation},
    InquireError, MultiSelect,
};

use super::{actions::MultiSelectPromptAction, config::MultiSelectConfig};

pub struct MultiSelectPrompt<'a, T> {
    message: &'a str,
    config: MultiSelectConfig,
    options: Vec<T>,
    string_options: Vec<String>,
    help_message: Option<&'a str>,
    cursor_index: usize,
    checked: BTreeSet<usize>,
    input: Input,
    filtered_options: Vec<usize>,
    filter: Filter<'a, T>,
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
        if let Some(default) = mso.default {
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
        let filtered_options = (0..mso.options.len()).collect();
        let checked_options = mso
            .default
            .map_or_else(BTreeSet::new, |d| d.iter().cloned().collect());

        Ok(Self {
            message: mso.message,
            config: (&mso).into(),
            options: mso.options,
            string_options,
            filtered_options,
            help_message: mso.help_message,
            cursor_index: mso.starting_cursor,
            input: Input::new(),
            filter: mso.filter,
            formatter: mso.formatter,
            validator: mso.validator,
            error: None,
            checked: checked_options,
        })
    }

    fn filter_options(&self) -> Vec<usize> {
        self.options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| match self.input.content() {
                val if val.is_empty() => Some(i),
                val if (self.filter)(val, opt, self.string_options.get(i).unwrap(), i) => Some(i),
                _ => None,
            })
            .collect()
    }

    fn move_cursor_up(&mut self, qty: usize, wrap: bool) -> HandleResult {
        let new_position = if wrap {
            let after_wrap = qty.saturating_sub(self.cursor_index);
            self.cursor_index
                .checked_sub(qty)
                .unwrap_or_else(|| self.filtered_options.len().saturating_sub(after_wrap))
        } else {
            self.cursor_index.saturating_sub(qty)
        };

        self.update_cursor_position(new_position)
    }

    fn move_cursor_down(&mut self, qty: usize, wrap: bool) -> HandleResult {
        let mut new_position = self.cursor_index.saturating_add(qty);

        if new_position >= self.filtered_options.len() {
            new_position = if self.filtered_options.is_empty() {
                0
            } else if wrap {
                new_position % self.filtered_options.len()
            } else {
                self.filtered_options.len().saturating_sub(1)
            }
        }

        self.update_cursor_position(new_position)
    }

    fn update_cursor_position(&mut self, new_position: usize) -> HandleResult {
        if new_position != self.cursor_index {
            self.cursor_index = new_position;
            HandleResult::Dirty
        } else {
            HandleResult::Clean
        }
    }

    fn toggle_cursor_selection(&mut self) -> HandleResult {
        let idx = match self.filtered_options.get(self.cursor_index) {
            Some(val) => val,
            None => return HandleResult::Clean,
        };

        if self.checked.contains(idx) {
            self.checked.remove(idx);
        } else {
            self.checked.insert(*idx);
        }

        if !self.config.keep_filter {
            self.input.clear();
        }

        HandleResult::Dirty
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
}

impl<'a, B, T> Prompt<B, MultiSelectConfig, MultiSelectPromptAction, Vec<ListOption<T>>>
    for MultiSelectPrompt<'a, T>
where
    B: MultiSelectBackend,
    T: Display,
{
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

    fn handle(&mut self, action: MultiSelectPromptAction) -> HandleResult {
        match action {
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
                for idx in &self.filtered_options {
                    self.checked.insert(*idx);
                }

                if !self.config.keep_filter {
                    self.input.clear();
                }

                HandleResult::Dirty
            }
            MultiSelectPromptAction::ClearSelections => {
                self.checked.clear();

                if !self.config.keep_filter {
                    self.input.clear();
                }

                HandleResult::Dirty
            }
            MultiSelectPromptAction::FilterInput(input_action) => {
                let result = self.input.handle(input_action);

                if let HandleResult::Dirty = result {
                    let options = self.filter_options();
                    self.filtered_options = options;
                    if self.filtered_options.len() <= self.cursor_index {
                        let _ = self
                            .update_cursor_position(self.filtered_options.len().saturating_sub(1));
                    }
                }

                result
            }
        }
    }

    fn render(&self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        backend.render_multiselect_prompt(prompt, &self.input)?;

        let choices = self
            .filtered_options
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
