#![allow(missing_docs)]

#[cfg(test)]
#[cfg(feature = "reorder")]
mod test;

use crate::{
    config::get_configuration,
    error::{InquireResult, InquireError},
    input::Input,
    list_option::ListOption,
    prompts::prompt::{Prompt, ActionResult},
    terminal::get_default_terminal,
    ui::{Backend, ReorderBackend, Key, KeyModifiers, RenderConfig},
    utils::paginate,
    InnerAction, InputAction,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReorderableListAction {
    FilterInput(InputAction),
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    MoveToStart,
    MoveToEnd,
    MoveItemUp,
    MoveItemDown,
}

impl InnerAction for ReorderableListAction {
    type Config = ReorderableListConfig;

    fn from_key(key: Key, config: &ReorderableListConfig) -> Option<Self> {
        if config.vim_mode {
            let action = match key {
                Key::Char('k', KeyModifiers::NONE) => Some(Self::MoveUp),
                Key::Char('j', KeyModifiers::NONE) => Some(Self::MoveDown),
                Key::Char('K', KeyModifiers::SHIFT) => Some(Self::MoveItemUp),
                Key::Char('J', KeyModifiers::SHIFT) => Some(Self::MoveItemDown),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        let action = match key {
            Key::Up(KeyModifiers::NONE) | Key::Char('p', KeyModifiers::CONTROL) => Self::MoveUp,
            Key::Down(KeyModifiers::NONE) | Key::Char('n', KeyModifiers::CONTROL) => Self::MoveDown,
            Key::PageUp(_) => Self::PageUp,
            Key::PageDown(_) => Self::PageDown,
            Key::Home => Self::MoveToStart,
            Key::End => Self::MoveToEnd,
            Key::Up(KeyModifiers::CONTROL) => Self::MoveItemUp,
            Key::Down(KeyModifiers::CONTROL) => Self::MoveItemDown,
            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::FilterInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ReorderableListConfig {
    pub vim_mode: bool,
    pub page_size: usize,
    pub keep_filter: bool,
    pub reset_cursor: bool,
}

pub struct ReorderableList<'a> {
    pub message: &'a str,
    pub options: Vec<String>,
    pub help_message: Option<&'a str>,
    pub page_size: usize,
    pub vim_mode: bool,
    pub starting_cursor: usize,
    pub starting_filter_input: Option<&'a str>,
    pub reset_cursor: bool,
    pub filter_input_enabled: bool,
    pub keep_filter: bool,
    pub formatter: &'a dyn Fn(&[String]) -> String,
    pub render_config: RenderConfig<'a>,
}

impl<'a> ReorderableList<'a> 
{
    pub fn new(message: &'a str, options: Vec<String>) -> Self {
        Self {
            message,
            options,
            help_message: Some("↑↓ to move cursor, Ctrl+↑↓ to move item, type to filter"),
            page_size: 7,
            vim_mode: false,
            starting_cursor: 0,
            starting_filter_input: None,
            reset_cursor: true,
            filter_input_enabled: true,
            keep_filter: true,
            formatter: &|items| items.join(", "),
            render_config: get_configuration(),
        }
    }

    pub fn with_render_config(mut self, render_config: RenderConfig<'a>) -> Self {
        self.render_config = render_config;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    ///
    /// Returns the owned objects selected by the user.
    pub fn prompt(self) -> InquireResult<Vec<String>> {
        self.raw_prompt()
            .map(|op| op.into_iter().collect())
    }

    pub fn prompt_skippable(self) -> InquireResult<Option<Vec<String>>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn raw_prompt_skippable(self) -> InquireResult<Option<Vec<String>>> {
        match self.raw_prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn raw_prompt(self) -> InquireResult<Vec<String>> {
        let (input_reader, terminal) = get_default_terminal()?;
        let mut backend = Backend::new(input_reader, terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }
    
    pub fn prompt_with_backend<B: ReorderBackend>(self, backend: &mut B) -> InquireResult<Vec<String>> {
        ReorderableListPrompt::new(self)?.prompt(backend)
    }
}

pub struct ReorderableListPrompt<'a> {
    message: &'a str,
    config: ReorderableListConfig,
    options: Vec<String>,
    help_message: Option<&'a str>,
    cursor_index: usize,
    input: Option<Input>,
    scored_options: Vec<usize>,
    formatter: &'a dyn Fn(&[String]) -> String,
}

impl<'a> ReorderableListPrompt<'a>
{
    pub fn new(rlo: ReorderableList<'a>) -> InquireResult<Self> {
        if rlo.options.is_empty() {
            return Err(InquireError::InvalidConfiguration(
                "Available options can not be empty".into(),
            ));
        }

        let scored_options = (0..rlo.options.len()).collect();

        let input = match rlo.filter_input_enabled {
            true => Some(Input::new_with(rlo.starting_filter_input.unwrap_or_default())),
            false => None,
        };

        Ok(Self {
            message: rlo.message,
            config: ReorderableListConfig {
                vim_mode: rlo.vim_mode,
                page_size: rlo.page_size,
                keep_filter: rlo.keep_filter,
                reset_cursor: rlo.reset_cursor,
            },
            options: rlo.options,
            scored_options,
            help_message: rlo.help_message,
            cursor_index: rlo.starting_cursor,
            input,
            formatter: rlo.formatter,
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

    fn move_item_up(&mut self) -> ActionResult {
        if self.cursor_index > 0 && !self.scored_options.is_empty() {
            self.scored_options.swap(self.cursor_index, self.cursor_index - 1);
            self.cursor_index -= 1;
            
            // Force a redraw of the frame
            return ActionResult::NeedsRedraw;
        }
        ActionResult::Clean
    }

    fn move_item_down(&mut self) -> ActionResult {
        if self.cursor_index < self.scored_options.len() - 1 {
            self.scored_options.swap(self.cursor_index, self.cursor_index + 1);
            self.cursor_index += 1;
            
            // Force a redraw of the frame
            return ActionResult::NeedsRedraw;
        }
        ActionResult::Clean
    }
    
    fn get_final_order(&mut self) -> Vec<String> {
        let mut reordered = Vec::with_capacity(self.options.len());
        
        // Create the final order based on scored_options
        for &idx in &self.scored_options {
            reordered.push(self.options[idx].clone());
        }
        
        reordered
    }
}

impl<'a, Backend> Prompt<Backend> for ReorderableListPrompt<'a>
where
    Backend: ReorderBackend,
{
    type Config = ReorderableListConfig;
    type InnerAction = ReorderableListAction;
    type Output = Vec<String>;

    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &ReorderableListConfig {
        &self.config
    }

    fn format_answer(&self, answer: &Vec<String>) -> String {
        (self.formatter)(answer)
    }

    fn handle(&mut self, action: ReorderableListAction) -> InquireResult<ActionResult> {
        let result = match action {
            ReorderableListAction::MoveUp => self.move_cursor_up(1, true),
            ReorderableListAction::MoveDown => self.move_cursor_down(1, true),
            ReorderableListAction::PageUp => self.move_cursor_up(self.config.page_size, false),
            ReorderableListAction::PageDown => self.move_cursor_down(self.config.page_size, false),
            ReorderableListAction::MoveToStart => self.move_cursor_up(usize::MAX, false),
            ReorderableListAction::MoveToEnd => self.move_cursor_down(usize::MAX, false),
            ReorderableListAction::MoveItemUp => self.move_item_up(),
            ReorderableListAction::MoveItemDown => self.move_item_down(),
            ReorderableListAction::FilterInput(input_action) => match self.input.as_mut() {
                Some(input) => {
                    let result = input.handle(input_action);
                    result.into()
                }
                None => ActionResult::Clean,
            },
        };

        Ok(result)
    }

    fn submit(&mut self) -> InquireResult<Option<Vec<String>>> {
        Ok(Some(self.get_final_order()))
    }

    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        // Clear the previous frame first
        backend.frame_setup()?;
        
        // Render the prompt
        backend.render_reorderable_prompt(&self.message, self.input.as_ref())?;

        // Create the list of visible options
        let choices = self.scored_options
            .iter()
            .enumerate()
            .map(|(i, &idx)| ListOption::new(i, &self.options[idx]))
            .collect::<Vec<_>>();

        // Calculate pagination
        let page = paginate(self.config.page_size, &choices, Some(self.cursor_index));
        
        // Render the options
        backend.render_reorder_options(page)?;

        // Render help message if present
        if let Some(help_message) = self.help_message {
            backend.render_help_message(help_message)?;
        }

        // Finish the frame
        backend.frame_finish(false)?;

        Ok(())
    }
}
