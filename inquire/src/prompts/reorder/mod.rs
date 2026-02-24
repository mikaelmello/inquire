//! Reorderable list prompt implementation.
//!
//! This module provides functionality for creating interactive prompts that allow users
//! to reorder a list of items. Users can navigate through the list and change the order
//! of items using keyboard controls.
//!
//! # Features
//!
//! - **Interactive reordering**: Move items up and down in the list
//! - **Keyboard navigation**: Arrow keys, page up/down, home/end support
//! - **Vim-style controls**: Optional hjkl navigation and JK for item movement
//! - **Filtering**: Optional real-time filtering of items by typing
//! - **Pagination**: Large lists are automatically paginated for better UX
//! - **Customizable styling**: Full control over colors and rendering
//!
//! # Controls
//!
//! ## Standard Mode
//! - `↑/↓` or `Ctrl+P/N`: Move cursor up/down
//! - `Ctrl+↑/↓`: Move selected item up/down
//! - `Page Up/Down`: Move cursor by page size
//! - `Home/End`: Move cursor to start/end
//! - `Enter`: Submit current order
//! - `Esc`: Cancel prompt
//! - `Type`: Filter items (if enabled)
//!
//! ## Vim Mode (when enabled)
//! - `j/k`: Move cursor down/up
//! - `J/K` (Shift): Move selected item down/up
//! - All other standard controls remain available
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use inquire::Reorder;
//!
//! let options = vec!["First", "Second", "Third", "Fourth"];
//! let result = Reorder::new("Please reorder these items:", options)
//!     .prompt();
//!
//! match result {
//!     Ok(reordered) => {
//!         println!("New order:");
//!         for (i, item) in reordered.iter().enumerate() {
//!             println!("{}. {}", i + 1, item);
//!         }
//!     }
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```
//!
//! ## With Custom Configuration
//!
//! ```no_run
//! use inquire::{Reorder, ui::RenderConfig};
//!
//! let options = vec!["Task A", "Task B", "Task C"];
//! let result = Reorder::new("Prioritize tasks:", options)
//!     .with_page_size(10)
//!     .with_vim_mode(true)
//!     .with_help_message("Use J/K to move items, j/k to navigate")
//!     .prompt();
//! ```
//!
//! # Implementation Details
//!
//! The reorder prompt is built on top of the general prompt framework and consists of:
//!
//! - [`ReorderAction`]: Enumeration of all possible user actions
//! - [`ReorderConfig`]: Configuration options for behavior
//! - [`Reorder`]: High-level builder and configuration struct
//! - [`ReorderPrompt`]: Internal prompt implementation handling state and logic
//!
//! The prompt maintains an internal mapping of display positions to original indices,
//! allowing efficient reordering without moving the actual items until submission.

mod prompt;
#[cfg(test)]
mod test;

use std::fmt::Display;

use crate::{
    config::get_configuration,
    error::{InquireError, InquireResult},
    prompts::prompt::Prompt,
    terminal::get_default_terminal,
    ui::{Backend, Key, KeyModifiers, RenderConfig, ReorderBackend},
    InnerAction, InputAction,
};

pub use prompt::ReorderPrompt;

/// Actions that can be performed in a reorderable list prompt.
///
/// This enum represents all possible user interactions with the reorderable list,
/// including navigation actions (moving the cursor) and reordering actions (moving items).
/// Each action corresponds to specific keyboard inputs as defined by the key mapping logic.
///
/// # Action Categories
///
/// ## Navigation Actions
/// These actions move the cursor without changing item order:
/// - `MoveUp`, `MoveDown`: Single-step cursor movement
/// - `PageUp`, `PageDown`: Multi-step cursor movement
/// - `MoveToStart`, `MoveToEnd`: Jump to list boundaries
///
/// ## Reordering Actions
/// These actions change the order of items:
/// - `MoveItemUp`, `MoveItemDown`: Move the selected item up or down
///
/// ## Input Actions
/// - `FilterInput`: Handle text input for filtering items
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReorderAction {
    /// Delegates to the filter input handler for text processing.
    ///
    /// This action is triggered when the user types characters to filter
    /// the list of items. The wrapped `InputAction` contains the specific
    /// input operation (character insertion, deletion, etc.).
    FilterInput(InputAction),

    /// Moves the cursor to highlight the option above the current selection.
    ///
    /// Triggered by: `↑`, `Ctrl+P`, or `k` (in vim mode)
    /// Wraps around to the bottom when at the top of the list.
    MoveUp,

    /// Moves the cursor to highlight the option below the current selection.
    ///
    /// Triggered by: `↓`, `Ctrl+N`, or `j` (in vim mode)
    /// Wraps around to the top when at the bottom of the list.
    MoveDown,

    /// Moves the cursor up by one page of items.
    ///
    /// Triggered by: `Page Up`
    /// The page size is configurable and defaults to 7 items.
    PageUp,

    /// Moves the cursor down by one page of items.
    ///
    /// Triggered by: `Page Down`
    /// The page size is configurable and defaults to 7 items.
    PageDown,

    /// Moves the cursor to the first item in the list.
    ///
    /// Triggered by: `Home`
    MoveToStart,

    /// Moves the cursor to the last item in the list.
    ///
    /// Triggered by: `End`
    MoveToEnd,

    /// Moves the currently selected item up one position in the list.
    ///
    /// Triggered by: `Ctrl+↑` or `K` (Shift+K in vim mode)
    /// Also moves the cursor up to follow the moved item.
    /// No effect if the item is already at the top.
    MoveItemUp,

    /// Moves the currently selected item down one position in the list.
    ///
    /// Triggered by: `Ctrl+↓` or `J` (Shift+J in vim mode)
    /// Also moves the cursor down to follow the moved item.
    /// No effect if the item is already at the bottom.
    MoveItemDown,
}

impl InnerAction for ReorderAction {
    type Config = ReorderConfig;

    fn from_key(key: Key, config: &ReorderConfig) -> Option<Self> {
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

/// Configuration settings for reorderable list prompt behavior.
///
/// This struct controls various aspects of how the reorderable list prompt
/// behaves and responds to user input. It allows customization of navigation
/// modes, pagination, and cursor behavior during filtering operations.
///
/// # Examples
///
/// ```no_run
/// use inquire::ReorderConfig;
///
/// let config = ReorderConfig {
///     vim_mode: true,        // Enable vim-style navigation
///     page_size: 10,         // Show 10 items per page
///     reset_cursor: false,   // Keep cursor position when filtering
/// };
/// ```
#[derive(Copy, Clone, Debug)]
pub struct ReorderConfig {
    /// Whether to enable vim-style keybindings for navigation and reordering.
    ///
    /// When enabled:
    /// - `j/k`: Move cursor down/up (equivalent to arrow keys)
    /// - `J/K` (Shift): Move selected item down/up (equivalent to Ctrl+arrows)
    /// - All standard keybindings remain available
    ///
    /// Default: `false`
    pub vim_mode: bool,

    /// Number of options to display per page.
    ///
    /// Large lists are automatically paginated to improve user experience.
    /// Users can navigate between pages using Page Up/Down keys.
    /// Setting this to a larger value shows more items at once but may
    /// cause issues on small terminal windows.
    ///
    /// Default: `7`
    pub page_size: usize,

    /// Whether to reset cursor position to the first item when filter input changes.
    ///
    /// When `true`: Cursor jumps to the first item after each filter keystroke
    /// When `false`: Cursor maintains its relative position in the filtered list
    ///
    /// Default: `true`
    pub reset_cursor: bool,
}

/// Prompt suitable for when you need the user to reorder options among many.
///
/// The user can select and move the current highlighted option by pressing Ctrl+Up or Ctrl+Down.
///
/// This prompt does not support custom validators because of its nature. It always returns all given options in the order they were reordered.
///
/// The options are paginated in order to provide a smooth experience to the user, with the default page size being 7. The user can move from the options and the pages will be updated accordingly, including moving from the last to the first options (or vice-versa).
///
/// Like all others, this prompt also allows you to customize several aspects of it:
///
/// - **Prompt message**: Required when creating the prompt.
/// - **Options list**: Options displayed to the user. Must be **non-empty**.
/// - **Starting cursor**: Index of the cursor when the prompt is first rendered. Default is 0 (first option). If the index is out-of-range of the option list, the prompt will fail with an [`InquireError::InvalidConfiguration`] error.
/// - **Starting filter input**: Sets the initial value of the filter section of the prompt.
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
///   - Prints the selected option string value by default.
/// - **Page size**: Number of options displayed at once, 7 by default.
///
/// # Example
///
/// ```no_run
/// use inquire::{error::InquireError, Reorder};
///
/// let options: Vec<&str> = vec!["Banana", "Apple", "Strawberry", "Grapes",
///     "Lemon", "Tangerine", "Watermelon", "Orange", "Pear", "Avocado", "Pineapple",
/// ];
///
/// let ans: Result<Vec<&str>, InquireError> = Reorder::new("Please, order your favorite fruits:", options).prompt();
///
/// println!("Fruit order: {:?}", ans.unwrap());
/// ```
///
/// [`InquireError::InvalidConfiguration`]: crate::error::InquireError::InvalidConfiguration
#[derive(Clone)]
pub struct Reorder<'a, T> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Options displayed to the user.
    pub options: Vec<T>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Page size of the options displayed to the user.
    pub page_size: usize,

    /// Whether vim mode is enabled. When enabled, the user can
    /// navigate through the options using hjkl.
    pub vim_mode: bool,

    /// Starting cursor index of the selection.
    pub starting_cursor: usize,

    /// Starting filter input
    pub starting_filter_input: Option<&'a str>,

    /// Reset cursor position to first option on filter input change.
    /// Defaults to true.
    pub reset_cursor: bool,

    /// Whether to allow the option list to be filtered by user input or not.
    ///
    /// Defaults to true.
    pub filter_input_enabled: bool,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: &'a dyn Fn(&[T]) -> String,

    /// RenderConfig to apply to the rendered interface.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still support NO_COLOR, you will have to do this on your end.
    pub render_config: RenderConfig<'a>,
}

impl<'a, T> Reorder<'a, T>
where
    T: Display + Clone,
{
    /// Default formatter that joins items with ", ".
    pub const DEFAULT_FORMATTER: &'a dyn Fn(&[T]) -> String = &|items| {
        items
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    };

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("↑↓ to move cursor, Ctrl+↑↓ to move item, type to filter");

    /// Default page size.
    pub const DEFAULT_PAGE_SIZE: usize = 7;

    /// Default vim mode setting.
    pub const DEFAULT_VIM_MODE: bool = false;

    /// Default starting cursor position.
    pub const DEFAULT_STARTING_CURSOR: usize = 0;

    /// Default cursor reset behavior.
    pub const DEFAULT_RESET_CURSOR: bool = true;

    /// Default filter input enabled setting.
    pub const DEFAULT_FILTER_INPUT_ENABLED: bool = true;

    /// Creates a new `Reorder` prompt with default settings.
    ///
    /// # Arguments
    ///
    /// * `message` - The prompt message to display to the user
    /// * `options` - Vector of items that can be reordered
    ///
    /// # Returns
    ///
    /// A new `Reorder` instance with default configuration:
    /// - Page size: 7 items
    /// - Vim mode: disabled
    /// - Filtering: enabled
    /// - Help message: shows basic controls
    /// - Default formatter: joins items with ", "
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Apple", "Banana", "Cherry"];
    /// let prompt = Reorder::new("Arrange fruits:", options);
    /// ```
    pub fn new(message: &'a str, options: Vec<T>) -> Self {
        Self {
            message,
            options,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            starting_cursor: Self::DEFAULT_STARTING_CURSOR,
            starting_filter_input: None,
            reset_cursor: Self::DEFAULT_RESET_CURSOR,
            filter_input_enabled: Self::DEFAULT_FILTER_INPUT_ENABLED,
            formatter: Self::DEFAULT_FORMATTER,
            render_config: get_configuration(),
        }
    }

    /// Sets the render configuration for styling and colors.
    ///
    /// # Arguments
    ///
    /// * `render_config` - The render configuration to use
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::{Reorder, ui::RenderConfig};
    ///
    /// let config = RenderConfig::default();
    /// let options = vec!["Item 1".to_string(), "Item 2".to_string()];
    /// let prompt = Reorder::new("Reorder:", options)
    ///     .with_render_config(config);
    /// ```
    pub fn with_render_config(mut self, render_config: RenderConfig<'a>) -> Self {
        self.render_config = render_config;
        self
    }

    /// Sets the help message displayed below the options.
    ///
    /// # Arguments
    ///
    /// * `message` - The help message to display
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Item 1", "Item 2"];
    /// let prompt = Reorder::new("Reorder:", options)
    ///     .with_help_message("Use Ctrl+arrows to move items");
    /// ```
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Removes the help message.
    pub fn without_help_message(mut self) -> Self {
        self.help_message = None;
        self
    }

    /// Sets the page size for pagination.
    ///
    /// # Arguments
    ///
    /// * `page_size` - Number of items to display per page
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Item 1", "Item 2"];
    /// let prompt = Reorder::new("Reorder:", options)
    ///     .with_page_size(10);
    /// ```
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Enables or disables vim-style keybindings.
    ///
    /// # Arguments
    ///
    /// * `vim_mode` - Whether to enable vim mode
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Item 1", "Item 2"];
    /// let prompt = Reorder::new("Reorder:", options)
    ///     .with_vim_mode(true);
    /// ```
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Sets the starting cursor position.
    ///
    /// # Arguments
    ///
    /// * `starting_cursor` - Index of the initially selected item
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Item 1", "Item 2", "Item 3"];
    /// let prompt = Reorder::new("Reorder:", options)
    ///     .with_starting_cursor(1); // Start at second item
    /// ```
    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Self {
        self.starting_cursor = starting_cursor;
        self
    }

    /// Sets the initial filter input value.
    ///
    /// # Arguments
    ///
    /// * `starting_filter_input` - Initial text for the filter
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Apple", "Banana", "Cherry"];
    /// let prompt = Reorder::new("Reorder fruits:", options)
    ///     .with_starting_filter_input("A"); // Pre-filter for items starting with 'A'
    /// ```
    pub fn with_starting_filter_input(mut self, starting_filter_input: &'a str) -> Self {
        self.starting_filter_input = Some(starting_filter_input);
        self
    }

    /// Sets the cursor reset behavior when filtering.
    ///
    /// # Arguments
    ///
    /// * `reset_cursor` - Whether to reset cursor to first item when filter changes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Item 1", "Item 2"];
    /// let prompt = Reorder::new("Reorder:", options)
    ///     .with_reset_cursor(false); // Keep cursor position when filtering
    /// ```
    pub fn with_reset_cursor(mut self, reset_cursor: bool) -> Self {
        self.reset_cursor = reset_cursor;
        self
    }

    /// Executes the prompt using a custom backend.
    ///
    /// This method allows using a custom backend implementation for testing
    /// or alternative rendering approaches.
    ///
    /// # Arguments
    ///
    /// * `backend` - The backend to use for rendering and input
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<T>)` - The reordered list of items
    /// * `Err(InquireError)` - If an error occurs
    pub fn prompt_with_backend<B: ReorderBackend>(self, backend: &mut B) -> InquireResult<Vec<T>> {
        ReorderPrompt::new(self)?.prompt(backend)
    }

    /// Disables the filter input functionality.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Item 1", "Item 2"];
    /// let prompt = Reorder::new("Reorder:", options)
    ///     .without_filtering(); // Disable typing to filter
    /// ```
    pub fn without_filtering(mut self) -> Self {
        self.filter_input_enabled = false;
        self
    }

    /// Sets a custom formatter for the final answer display.
    ///
    /// # Arguments
    ///
    /// * `formatter` - Function to format the reordered list for display
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["Item 1", "Item 2"];
    /// let prompt = Reorder::new("Reorder:", options)
    ///     .with_formatter(&|items| format!("Selected: [{}]", items.join(" -> ")));
    /// ```
    pub fn with_formatter(mut self, formatter: &'a dyn Fn(&[T]) -> String) -> Self {
        self.formatter = formatter;
        self
    }

    /// Executes the prompt and returns the reordered list.
    ///
    /// This method displays the interactive prompt to the user and waits for them
    /// to reorder the items and submit their selection.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<T>)` - The reordered list of items
    /// * `Err(InquireError)` - If an error occurs or the user cancels
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["First", "Second"];
    /// let result = Reorder::new("Reorder items:", options)
    ///     .prompt();
    ///
    /// match result {
    ///     Ok(reordered) => println!("New order: {:?}", reordered),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    /// Executes the prompt and returns the reordered list.
    ///
    /// This method displays the interactive prompt to the user and waits for them
    /// to reorder the items and submit their selection.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<T>)` - The reordered list of items
    /// * `Err(InquireError)` - If an error occurs or the user cancels
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use inquire::Reorder;
    ///
    /// let options = vec!["First", "Second"];
    /// let result = Reorder::new("Reorder items:", options)
    ///     .prompt();
    ///
    /// match result {
    ///     Ok(reordered) => println!("New order: {:?}", reordered),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub fn prompt(self) -> InquireResult<Vec<T>> {
        self.raw_prompt()
    }

    /// Executes the prompt with the ability to skip/cancel.
    ///
    /// Like `prompt()`, but treats cancellation (ESC key) as a normal operation
    /// rather than an error, returning `Ok(None)` instead of `Err(OperationCanceled)`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Vec<T>))` - The reordered list if submitted
    /// * `Ok(None)` - If the user cancelled the prompt
    /// * `Err(InquireError)` - If an error occurs
    pub fn prompt_skippable(self) -> InquireResult<Option<Vec<T>>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Executes the prompt with raw output and the ability to skip/cancel.
    ///
    /// Similar to `prompt_skippable()` but returns the raw prompt result.
    /// This is the low-level interface that other methods build upon.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Vec<T>))` - The reordered list if submitted
    /// * `Ok(None)` - If the user cancelled the prompt
    /// * `Err(InquireError)` - If an error occurs
    pub fn raw_prompt_skippable(self) -> InquireResult<Option<Vec<T>>> {
        match self.raw_prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Executes the prompt and returns the raw result.
    ///
    /// This is the low-level interface that other prompt methods build upon.
    /// It handles the actual terminal interaction and user input processing.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<T>)` - The reordered list of items
    /// * `Err(InquireError)` - If an error occurs or the user cancels
    pub fn raw_prompt(self) -> InquireResult<Vec<T>> {
        let (input_reader, terminal) = get_default_terminal()?;
        let mut backend = Backend::new(input_reader, terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }
}
