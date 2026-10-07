//! Implementation of the reorderable list prompt.
//!
//! This module provides the core logic for handling user interactions with a reorderable list,
//! including cursor movement, item reordering, and rendering the prompt interface.

use std::fmt::Display;

use crate::{
    error::InquireResult,
    input::Input,
    list_option::ListOption,
    prompts::prompt::{ActionResult, Prompt},
    ui::ReorderBackend,
    utils::paginate,
    InquireError, Reorder, ReorderAction, ReorderConfig,
};

/// Internal state and logic for the reorderable list prompt.
///
/// This struct maintains the current state of a reorderable list prompt, including:
/// - The current order of items (tracked via `scored_options`)
/// - Cursor position within the list
/// - Filter input state (if filtering is enabled)
/// - Configuration options for behavior and rendering
///
/// The prompt allows users to:
/// - Navigate through items using arrow keys or vim-style navigation
/// - Reorder items using Ctrl+Up/Down or Shift+J/K (in vim mode)
/// - Filter items by typing (if enabled)
/// - Submit the final reordered list
///
/// # Generic Parameters
///
/// * `T` - The type of items in the list. Must implement `Display + Clone` for rendering and output.
pub struct ReorderPrompt<'a, T> {
    /// The prompt message displayed to the user
    message: &'a str,
    /// Configuration settings for the prompt behavior
    config: ReorderConfig,
    /// The original list of options provided by the user
    options: Vec<T>,
    /// Optional help message shown below the options
    help_message: Option<&'a str>,
    /// Current cursor position (index in the visible/filtered list)
    cursor_index: usize,
    /// Optional input handler for filtering (None if filtering is disabled)
    input: Option<Input>,
    /// Vector of indices representing the current order of items
    /// Maps display position -> original option index
    scored_options: Vec<usize>,
    /// Function to format the final answer for display
    formatter: &'a dyn Fn(&[T]) -> String,
}

impl<'a, T> ReorderPrompt<'a, T>
where
    T: Display + Clone,
{
    /// Creates a new `ReorderPrompt` from a `Reorder` configuration.
    ///
    /// This constructor validates the configuration and initializes the internal state:
    /// - Ensures the options list is not empty
    /// - Sets up the initial ordering (preserving original order)
    /// - Configures filter input if enabled
    /// - Validates the starting cursor position
    ///
    /// # Arguments
    ///
    /// * `rlo` - The `Reorder` configuration containing all prompt settings
    ///
    /// # Returns
    ///
    /// * `Ok(ReorderPrompt)` - Successfully created prompt
    /// * `Err(InquireError::InvalidConfiguration)` - If options list is empty
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::{Reorder, ReorderPrompt};
    ///
    /// let reorder = Reorder::new("Select order", vec!["First", "Second", "Third"]);
    /// let prompt = ReorderPrompt::new(reorder).unwrap();
    /// ```
    pub fn new(rlo: Reorder<'a, T>) -> InquireResult<Self> {
        if rlo.options.is_empty() {
            return Err(InquireError::InvalidConfiguration(
                "Available options can not be empty".into(),
            ));
        }

        let scored_options = (0..rlo.options.len()).collect();

        let input = match rlo.filter_input_enabled {
            true => Some(Input::new_with(
                rlo.starting_filter_input.unwrap_or_default(),
            )),
            false => None,
        };

        Ok(Self {
            message: rlo.message,
            config: ReorderConfig {
                vim_mode: rlo.vim_mode,
                page_size: rlo.page_size,
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

    /// Moves the cursor up by the specified quantity.
    ///
    /// # Arguments
    ///
    /// * `qty` - Number of positions to move up
    /// * `wrap` - Whether to wrap around to the bottom when reaching the top
    ///
    /// # Returns
    ///
    /// * `ActionResult::NeedsRedraw` - If the cursor position changed
    /// * `ActionResult::Clean` - If the cursor position remained the same
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

    /// Moves the cursor down by the specified quantity.
    ///
    /// # Arguments
    ///
    /// * `qty` - Number of positions to move down
    /// * `wrap` - Whether to wrap around to the top when reaching the bottom
    ///
    /// # Returns
    ///
    /// * `ActionResult::NeedsRedraw` - If the cursor position changed
    /// * `ActionResult::Clean` - If the cursor position remained the same
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

    /// Updates the cursor position and returns whether a redraw is needed.
    ///
    /// # Arguments
    ///
    /// * `new_position` - The new cursor position to set
    ///
    /// # Returns
    ///
    /// * `ActionResult::NeedsRedraw` - If the position changed
    /// * `ActionResult::Clean` - If the position remained the same
    fn update_cursor_position(&mut self, new_position: usize) -> ActionResult {
        if new_position != self.cursor_index {
            self.cursor_index = new_position;
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    /// Moves the currently selected item up one position in the list.
    ///
    /// This operation:
    /// - Swaps the current item with the one above it
    /// - Moves the cursor up to follow the moved item
    /// - Only works if not at the top of the list
    ///
    /// # Returns
    ///
    /// * `ActionResult::NeedsRedraw` - If an item was moved
    /// * `ActionResult::Clean` - If no movement was possible (already at top)
    fn move_item_up(&mut self) -> ActionResult {
        if self.cursor_index > 0 && !self.scored_options.is_empty() {
            self.scored_options
                .swap(self.cursor_index, self.cursor_index - 1);
            self.cursor_index -= 1;

            // Force a redraw of the frame
            return ActionResult::NeedsRedraw;
        }
        ActionResult::Clean
    }

    /// Moves the currently selected item down one position in the list.
    ///
    /// This operation:
    /// - Swaps the current item with the one below it
    /// - Moves the cursor down to follow the moved item
    /// - Only works if not at the bottom of the list
    ///
    /// # Returns
    ///
    /// * `ActionResult::NeedsRedraw` - If an item was moved
    /// * `ActionResult::Clean` - If no movement was possible (already at bottom)
    fn move_item_down(&mut self) -> ActionResult {
        if self.cursor_index < self.scored_options.len() - 1 {
            self.scored_options
                .swap(self.cursor_index, self.cursor_index + 1);
            self.cursor_index += 1;

            // Force a redraw of the frame
            return ActionResult::NeedsRedraw;
        }
        ActionResult::Clean
    }

    /// Generates the final reordered list based on the current item positions.
    ///
    /// This method clones items from the original options vector in the order
    /// specified by `scored_options`, creating the final output that reflects
    /// all reordering operations performed by the user.
    ///
    /// # Returns
    ///
    /// A new `Vec<T>` containing the items in their final reordered positions.
    fn get_final_order(&mut self) -> Vec<T> {
        let mut reordered = Vec::with_capacity(self.options.len());

        // Create the final order based on scored_options
        for &idx in &self.scored_options {
            reordered.push(self.options[idx].clone());
        }

        reordered
    }
}

/// Implementation of the `Prompt` trait for `ReorderPrompt`.
///
/// This implementation defines how the reorderable list prompt integrates with
/// the inquire prompt system, handling user input, rendering, and state management.
impl<'a, Backend, T> Prompt<Backend> for ReorderPrompt<'a, T>
where
    Backend: ReorderBackend,
    T: Display + Clone,
{
    type Config = ReorderConfig;
    type InnerAction = ReorderAction;
    type Output = Vec<T>;

    /// Returns the prompt message to be displayed to the user.
    fn message(&self) -> &str {
        self.message
    }

    /// Returns the configuration settings for this prompt.
    fn config(&self) -> &ReorderConfig {
        &self.config
    }

    /// Formats the final answer for display using the configured formatter.
    ///
    /// This is called when the prompt completes to show the user's final selection.
    fn format_answer(&self, answer: &Vec<T>) -> String {
        (self.formatter)(answer)
    }

    /// Handles user input actions and updates the prompt state accordingly.
    ///
    /// This method processes different types of user actions:
    /// - Navigation actions (move cursor up/down, page up/down, etc.)
    /// - Reordering actions (move item up/down)
    /// - Filter input actions (if filtering is enabled)
    ///
    /// # Arguments
    ///
    /// * `action` - The user action to process
    ///
    /// # Returns
    ///
    /// * `Ok(ActionResult)` - Indicates whether a redraw is needed
    /// * `Err(InquireError)` - If an error occurred processing the action
    fn handle(&mut self, action: ReorderAction) -> InquireResult<ActionResult> {
        let result = match action {
            ReorderAction::MoveUp => self.move_cursor_up(1, true),
            ReorderAction::MoveDown => self.move_cursor_down(1, true),
            ReorderAction::PageUp => self.move_cursor_up(self.config.page_size, false),
            ReorderAction::PageDown => self.move_cursor_down(self.config.page_size, false),
            ReorderAction::MoveToStart => self.move_cursor_up(usize::MAX, false),
            ReorderAction::MoveToEnd => self.move_cursor_down(usize::MAX, false),
            ReorderAction::MoveItemUp => self.move_item_up(),
            ReorderAction::MoveItemDown => self.move_item_down(),
            ReorderAction::FilterInput(input_action) => match self.input.as_mut() {
                Some(input) => {
                    let result = input.handle(input_action);
                    result.into()
                }
                None => ActionResult::Clean,
            },
        };

        Ok(result)
    }

    /// Handles prompt submission and returns the final reordered list.
    ///
    /// This method is called when the user presses Enter to submit their
    /// reordered list. It always returns the current order of items.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Vec<T>))` - The reordered list of items
    fn submit(&mut self) -> InquireResult<Option<Vec<T>>> {
        Ok(Some(self.get_final_order()))
    }

    /// Renders the prompt interface using the provided backend.
    ///
    /// This method coordinates the rendering of all prompt components:
    /// 1. The prompt message and filter input (if enabled)
    /// 2. The paginated list of reorderable options with cursor highlighting
    /// 3. The help message (if configured)
    ///
    /// The rendering respects the configured page size and shows appropriate
    /// visual indicators for the current cursor position and item order.
    ///
    /// # Arguments
    ///
    /// * `backend` - The rendering backend to use for output
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If rendering completed successfully
    /// * `Err(InquireError)` - If a rendering error occurred
    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        // Render the prompt
        backend.render_reorderable_prompt(&self.message, self.input.as_ref())?;

        // Create the list of visible options
        let choices = self
            .scored_options
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

        Ok(())
    }
}
