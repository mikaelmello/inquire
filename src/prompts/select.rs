use std::fmt::Display;

use crate::{
    config::{self, Filter},
    error::{InquireError, InquireResult},
    formatter::OptionFormatter,
    input::Input,
    list_option::ListOption,
    ui::{crossterm::CrosstermTerminal, Backend, Key, KeyModifiers, RenderConfig, SelectBackend},
    utils::paginate,
};

/// Prompt suitable for when you need the user to select one option among many.
///
/// The user can select and submit the current highlighted option by pressing enter.
///
/// This prompt requires a prompt message and a **non-empty** list of options to be displayed to the user. If the list is empty, the prompt operation will fail with an [`InquireError::InvalidConfiguration`] error.
///
/// This prompt does not support custom validators because of its nature. A submission always selects exactly one of the options. If this option was not supposed to be selected or is invalid in some way, it probably should not be included in the options list.
///
/// The options are paginated in order to provide a smooth experience to the user, with the default page size being 7. The user can move from the options and the pages will be updated accordingly, including moving from the last to the first options (or vice-versa).
///
/// Like all others, this prompt also allows you to customize several aspects of it:
///
/// - **Prompt message**: Required when creating the prompt.
/// - **Options list**: Options displayed to the user. Must be **non-empty**.
/// - **Starting cursor**: Index of the cursor when the prompt is first rendered. Default is 0 (first option). If the index is out-of-range of the option list, the prompt will fail with an [`InquireError::InvalidConfiguration`] error.
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
///   - Prints the selected option string value by default.
/// - **Page size**: Number of options displayed at once, 7 by default.
/// - **Filter function**: Function that defines if an option is displayed or not based on the current filter input.
///
/// # Example
///
/// ```no_run
/// use inquire::Select;
///
/// let options = vec!["Banana", "Apple", "Strawberry", "Grapes",
///     "Lemon", "Tangerine", "Watermelon", "Orange", "Pear", "Avocado", "Pineapple",
/// ];
///
/// let ans = Select::new("What's your favorite fruit?", &options).prompt();
///
/// match ans {
///     Ok(choice) => println!("{}! That's mine too!", choice.value),
///     Err(_) => println!("There was an error, please try again"),
/// }
/// ```
///
/// [`InquireError::InvalidConfiguration`]: crate::error::InquireError::InvalidConfiguration
#[derive(Clone)]
pub struct Select<'a, T> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Options displayed to the user.
    pub options: &'a [T],

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Page size of the options displayed to the user.
    pub page_size: usize,

    /// Whether vim mode is enabled. When enabled, the user can
    /// navigate through the options using hjkl.
    pub vim_mode: bool,

    /// Starting cursor index of the selection.
    pub starting_cursor: usize,

    /// Function called with the current user input to filter the provided
    /// options.
    pub filter: Filter<'a, T>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: OptionFormatter<'a, T>,

    /// RenderConfig to apply to the rendered interface.
    pub render_config: &'a RenderConfig,
}

impl<'a, T> Select<'a, T>
where
    T: Display,
{
    /// String formatter used by default in [Select](crate::Select) prompts.
    /// Simply prints the string value contained in the selected option.
    ///
    /// # Examples
    ///
    /// ```
    /// use inquire::list_option::ListOption;
    /// use inquire::Select;
    ///
    /// let formatter = Select::<&str>::DEFAULT_FORMATTER;
    /// assert_eq!(String::from("First option"), formatter(&ListOption::new(0, &"First option")));
    /// assert_eq!(String::from("First option"), formatter(&ListOption::new(11, &"First option")));
    /// ```
    pub const DEFAULT_FORMATTER: OptionFormatter<'a, T> = &|ans| ans.to_string();

    /// Default filter function, which checks if the current filter value is a substring of the option value.
    /// If it is, the option is displayed.
    ///
    /// # Examples
    ///
    /// ```
    /// use inquire::Select;
    ///
    /// let filter = Select::<&str>::DEFAULT_FILTER;
    /// assert_eq!(false, filter("sa", &"New York",      "New York",      0));
    /// assert_eq!(true,  filter("sa", &"Sacramento",    "Sacramento",    1));
    /// assert_eq!(true,  filter("sa", &"Kansas",        "Kansas",        2));
    /// assert_eq!(true,  filter("sa", &"Mesa",          "Mesa",          3));
    /// assert_eq!(false, filter("sa", &"Phoenix",       "Phoenix",       4));
    /// assert_eq!(false, filter("sa", &"Philadelphia",  "Philadelphia",  5));
    /// assert_eq!(true,  filter("sa", &"San Antonio",   "San Antonio",   6));
    /// assert_eq!(true,  filter("sa", &"San Diego",     "San Diego",     7));
    /// assert_eq!(false, filter("sa", &"Dallas",        "Dallas",        8));
    /// assert_eq!(true,  filter("sa", &"San Francisco", "San Francisco", 9));
    /// assert_eq!(false, filter("sa", &"Austin",        "Austin",       10));
    /// assert_eq!(false, filter("sa", &"Jacksonville",  "Jacksonville", 11));
    /// assert_eq!(true,  filter("sa", &"San Jose",      "San Jose",     12));
    /// ```
    pub const DEFAULT_FILTER: Filter<'a, T> = &|filter, _, string_value, _| -> bool {
        let filter = filter.to_lowercase();

        string_value.to_lowercase().contains(&filter)
    };

    /// Default page size.
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;

    /// Default value of vim mode.
    pub const DEFAULT_VIM_MODE: bool = config::DEFAULT_VIM_MODE;

    /// Default starting cursor index.
    pub const DEFAULT_STARTING_CURSOR: usize = 0;

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("↑↓ to move, enter to select, type to filter");

    /// Creates a [Select] with the provided message and options, along with default configuration values.
    pub fn new(message: &'a str, options: &'a [T]) -> Self {
        Self {
            message,
            options,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            starting_cursor: Self::DEFAULT_STARTING_CURSOR,
            filter: Self::DEFAULT_FILTER,
            formatter: Self::DEFAULT_FORMATTER,
            render_config: RenderConfig::default_static_ref(),
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Removes the set help message.
    pub fn without_help_message(mut self) -> Self {
        self.help_message = None;
        self
    }

    /// Sets the page size.
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Enables or disabled vim_mode.
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Sets the filter function.
    pub fn with_filter(mut self, filter: Filter<'a, T>) -> Self {
        self.filter = filter;
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: OptionFormatter<'a, T>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the starting cursor index.
    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Self {
        self.starting_cursor = starting_cursor;
        self
    }

    /// Sets the provided color theme to this prompt.
    pub fn with_render_config(mut self, render_config: &'a RenderConfig) -> Self {
        self.render_config = render_config;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<ListOption<&'a T>> {
        let terminal = CrosstermTerminal::new()?;
        let mut backend = Backend::new(terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(in crate) fn prompt_with_backend<B: SelectBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<ListOption<&'a T>> {
        SelectPrompt::new(self)?.prompt(backend)
    }
}

struct SelectPrompt<'a, T> {
    message: &'a str,
    options: &'a [T],
    string_options: Vec<String>,
    help_message: Option<&'a str>,
    vim_mode: bool,
    cursor_index: usize,
    page_size: usize,
    input: Input,
    filtered_options: Vec<usize>,
    filter: Filter<'a, T>,
    formatter: OptionFormatter<'a, T>,
}

impl<'a, T> SelectPrompt<'a, T>
where
    T: Display,
{
    fn new(so: Select<'a, T>) -> InquireResult<Self> {
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
        let filtered_options = (0..so.options.len()).collect();

        Ok(Self {
            message: so.message,
            options: so.options,
            string_options,
            filtered_options,
            help_message: so.help_message,
            vim_mode: so.vim_mode,
            cursor_index: so.starting_cursor,
            page_size: so.page_size,
            input: Input::new(),
            filter: so.filter,
            formatter: so.formatter,
        })
    }

    fn filter_options(&self) -> Vec<usize> {
        self.options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| match self.input.content() {
                val if val.is_empty() => Some(i),
                val if (self.filter)(&val, opt, self.string_options.get(i).unwrap(), i) => Some(i),
                _ => None,
            })
            .collect()
    }

    fn move_cursor_up(&mut self, qty: usize, wrap: bool) {
        if wrap {
            let after_wrap = qty.saturating_sub(self.cursor_index);
            self.cursor_index = self
                .cursor_index
                .checked_sub(qty)
                .unwrap_or(self.filtered_options.len().saturating_sub(after_wrap))
        } else {
            self.cursor_index = self.cursor_index.saturating_sub(qty);
        }
    }

    fn move_cursor_down(&mut self, qty: usize, wrap: bool) {
        self.cursor_index = self.cursor_index.saturating_add(qty);

        if self.cursor_index >= self.filtered_options.len() {
            self.cursor_index = if wrap {
                self.cursor_index % self.filtered_options.len()
            } else {
                self.filtered_options.len().saturating_sub(1)
            }
        }
    }

    fn on_change(&mut self, key: Key) {
        match key {
            Key::Up(KeyModifiers::NONE) => self.move_cursor_up(1, true),
            Key::Char('k', KeyModifiers::NONE) if self.vim_mode => self.move_cursor_up(1, true),
            Key::PageUp => self.move_cursor_up(self.page_size, false),
            Key::Home => self.move_cursor_up(usize::MAX, false),

            Key::Down(KeyModifiers::NONE) => self.move_cursor_down(1, true),
            Key::Char('j', KeyModifiers::NONE) if self.vim_mode => self.move_cursor_down(1, true),
            Key::PageDown => self.move_cursor_down(self.page_size, false),
            Key::End => self.move_cursor_down(usize::MAX, false),

            key => {
                let dirty = self.input.handle_key(key);

                if dirty {
                    let options = self.filter_options();
                    if options.len() > 0 && options.len() <= self.cursor_index {
                        self.cursor_index = options.len().saturating_sub(1);
                    }
                    self.filtered_options = options;
                }
            }
        };
    }

    fn get_final_answer(&mut self) -> ListOption<&'a T> {
        let index = *self.filtered_options.get(self.cursor_index).unwrap();
        let value = self.options.get(index).unwrap();

        ListOption::new(index, value)
    }

    fn render<B: SelectBackend>(&mut self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        backend.frame_setup()?;

        backend.render_select_prompt(&prompt, &self.input)?;

        let choices = self
            .filtered_options
            .iter()
            .cloned()
            .map(|i| ListOption::new(i, self.options.get(i).unwrap()))
            .collect::<Vec<ListOption<&T>>>();

        let page = paginate(self.page_size, &choices, self.cursor_index);

        backend.render_options(page)?;

        if let Some(help_message) = self.help_message {
            backend.render_help_message(help_message)?;
        }

        backend.frame_finish()?;

        Ok(())
    }

    fn prompt<B: SelectBackend>(mut self, backend: &mut B) -> InquireResult<ListOption<&'a T>> {
        loop {
            self.render(backend)?;

            let key = backend.read_key()?;

            match key {
                Key::Cancel => return Err(InquireError::OperationCanceled),
                Key::Submit => {
                    break;
                }
                key => self.on_change(key),
            }
        }
        let final_answer = self.get_final_answer();
        let formatted = (self.formatter)(&final_answer);

        backend.finish_prompt(&self.message, &formatted)?;

        Ok(final_answer)
    }
}
