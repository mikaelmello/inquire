mod action;
mod config;
mod prompt;
#[cfg(test)]
#[cfg(feature = "crossterm")]
mod test;

pub use action::*;

use std::fmt::Display;

use crate::{
    config::get_configuration,
    error::{InquireError, InquireResult},
    formatter::MultiOptionFormatter,
    list_option::ListOption,
    prompts::prompt::Prompt,
    terminal::get_default_terminal,
    type_aliases::Filter,
    ui::{Backend, MultiSelectBackend, RenderConfig},
    validator::MultiOptionValidator,
};

use self::prompt::MultiSelectPrompt;

/// Prompt suitable for when you need the user to select many options (including none if applicable) among a list of them.
///
/// The user can select (or deselect) the current highlighted option by pressing space, clean all selections by pressing the left arrow and select all options by pressing the right arrow.
///
/// This prompt requires a prompt message and a **non-empty** `Vec` of options to be displayed to the user. The options can be of any type as long as they implement the `Display` trait. It is required that the `Vec` is moved to the prompt, as the prompt will return the ownership of the `Vec` after the user submits, with only the selected options inside it.
/// - If the list is empty, the prompt operation will fail with an `InquireError::InvalidConfiguration` error.
///
/// The options are paginated in order to provide a smooth experience to the user, with the default page size being 7. The user can move from the options and the pages will be updated accordingly, including moving from the last to the first options (or vice-versa).
///
/// Customizable options:
///
/// - **Prompt message**: Required when creating the prompt.
/// - **Options list**: Options displayed to the user. Must be **non-empty**.
/// - **Default selections**: Options that are selected by default when the prompt is first rendered. The user can unselect them. If any of the indices is out-of-range of the option list, the prompt will fail with an [`InquireError::InvalidConfiguration`] error.
/// - **Starting cursor**: Index of the cursor when the prompt is first rendered. Default is 0 (first option). If the index is out-of-range of the option list, the prompt will fail with an [`InquireError::InvalidConfiguration`] error.
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
///   - Prints the selected options string value, joined using a comma as the separator, by default.
/// - **Validator**: Custom validator to make sure a given submitted input pass the specified requirements, e.g. not allowing 0 selected options or limiting the number of options that the user is allowed to select.
///   - No validators are on by default.
/// - **Page size**: Number of options displayed at once, 7 by default.
/// - **Display option indexes**: On long lists, it might be helpful to display the indexes of the options to the user. Via the `RenderConfig`, you can set the display mode of the indexes as a prefix of an option. The default configuration is `None`, to not render any index when displaying the options.
/// - **Filter function**: Function that defines if an option is displayed or not based on the current filter input.
/// - **Keep filter flag**: Whether the current filter input should be cleared or not after a selection is made. Defaults to true.
///
/// # Example
///
/// For a full-featured example, check the [GitHub repository](https://github.com/mikaelmello/inquire/blob/main/examples/multiselect.rs).
///
/// [`InquireError::InvalidConfiguration`]: crate::error::InquireError::InvalidConfiguration
#[derive(Clone)]
pub struct MultiSelect<'a, T> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Options displayed to the user.
    pub options: Vec<T>,

    /// Default indexes of options to be selected from the start.
    pub default: Option<&'a [usize]>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Initial value of the prompt's text input.
    pub initial_input: Option<&'a str>,

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

    /// Whether the current filter typed by the user is kept or cleaned after a selection is made.
    pub keep_filter: bool,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: MultiOptionFormatter<'a, T>,

    /// Validator to apply to the user input.
    ///
    /// In case of error, the message is displayed one line above the prompt.
    pub validator: Option<Box<dyn MultiOptionValidator<T>>>,

    /// RenderConfig to apply to the rendered interface.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub render_config: RenderConfig<'a>,
}

impl<'a, T> MultiSelect<'a, T>
where
    T: Display,
{
    /// String formatter used by default in [MultiSelect](crate::MultiSelect) prompts.
    /// Prints the string value of all selected options, separated by commas.
    ///
    /// # Examples
    ///
    /// ```
    /// use inquire::list_option::ListOption;
    /// use inquire::MultiSelect;
    ///
    /// let formatter = MultiSelect::<&str>::DEFAULT_FORMATTER;
    ///
    /// let mut ans = vec![ListOption::new(0, &"New York")];
    /// assert_eq!(String::from("New York"), formatter(&ans));
    ///
    /// ans.push(ListOption::new(3, &"Seattle"));
    /// assert_eq!(String::from("New York, Seattle"), formatter(&ans));
    ///
    /// ans.push(ListOption::new(7, &"Vancouver"));
    /// assert_eq!(String::from("New York, Seattle, Vancouver"), formatter(&ans));
    /// ```
    pub const DEFAULT_FORMATTER: MultiOptionFormatter<'a, T> = &|ans| {
        ans.iter()
            .map(|opt| opt.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    };

    /// Default filter function, which checks if the current filter value is a substring of the option value.
    /// If it is, the option is displayed.
    ///
    /// # Examples
    ///
    /// ```
    /// use inquire::MultiSelect;
    ///
    /// let filter = MultiSelect::<&str>::DEFAULT_FILTER;
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

    /// Default page size, equal to the global default page size [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_PAGE_SIZE: usize = crate::config::DEFAULT_PAGE_SIZE;

    /// Default value of vim mode, equal to the global default value [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_VIM_MODE: bool = crate::config::DEFAULT_VIM_MODE;

    /// Default starting cursor index.
    pub const DEFAULT_STARTING_CURSOR: usize = 0;

    /// Default behavior of keeping or cleaning the current filter value.
    pub const DEFAULT_KEEP_FILTER: bool = true;

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("↑↓ to move, space to select one, → to all, ← to none, type to filter");

    /// Creates a [MultiSelect] with the provided message and options, along with default configuration values.
    pub fn new(message: &'a str, options: Vec<T>) -> Self {
        Self {
            message,
            options,
            default: None,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            initial_input: None,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            starting_cursor: Self::DEFAULT_STARTING_CURSOR,
            keep_filter: Self::DEFAULT_KEEP_FILTER,
            filter: Self::DEFAULT_FILTER,
            formatter: Self::DEFAULT_FORMATTER,
            validator: None,
            render_config: get_configuration(),
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Sets the predefined text for the prompts text input
    pub fn with_initial_input(mut self, initial_input: &'a str) -> Self {
        self.initial_input = Some(initial_input);
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

    /// Enables or disables vim_mode.
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Sets the keep filter behavior.
    pub fn with_keep_filter(mut self, keep_filter: bool) -> Self {
        self.keep_filter = keep_filter;
        self
    }

    /// Sets the filter function.
    pub fn with_filter(mut self, filter: Filter<'a, T>) -> Self {
        self.filter = filter;
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: MultiOptionFormatter<'a, T>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the validator to apply to the user input. You might want to use this feature
    /// in case you need to limit the user to specific choices, such as limiting the number
    /// of selections.
    ///
    /// In case of error, the message is displayed one line above the prompt.
    pub fn with_validator<V>(mut self, validator: V) -> Self
    where
        V: MultiOptionValidator<T> + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    /// Sets the indexes to be selected by the default.
    pub fn with_default(mut self, default: &'a [usize]) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets the starting cursor index.
    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Self {
        self.starting_cursor = starting_cursor;
        self
    }

    /// Sets the provided color theme to this prompt.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub fn with_render_config(mut self, render_config: RenderConfig<'a>) -> Self {
        self.render_config = render_config;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    ///
    /// Returns the owned objects selected by the user.
    ///
    /// This method is intended for flows where the user skipping/cancelling
    /// the prompt - by pressing ESC - is considered normal behavior. In this case,
    /// it does not return `Err(InquireError::OperationCanceled)`, but `Ok(None)`.
    ///
    /// Meanwhile, if the user does submit an answer, the method wraps the return
    /// type with `Some`.
    pub fn prompt_skippable(self) -> InquireResult<Option<Vec<T>>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    ///
    /// Returns the owned objects selected by the user.
    pub fn prompt(self) -> InquireResult<Vec<T>> {
        self.raw_prompt()
            .map(|op| op.into_iter().map(|o| o.value).collect())
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    ///
    /// Returns a vector of [`ListOption`](crate::list_option::ListOption)s containing
    /// the index of the selections and the owned objects selected by the user.
    ///
    /// This method is intended for flows where the user skipping/cancelling
    /// the prompt - by pressing ESC - is considered normal behavior. In this case,
    /// it does not return `Err(InquireError::OperationCanceled)`, but `Ok(None)`.
    ///
    /// Meanwhile, if the user does submit an answer, the method wraps the return
    /// type with `Some`.
    pub fn raw_prompt_skippable(self) -> InquireResult<Option<Vec<ListOption<T>>>> {
        match self.raw_prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    ///
    /// Returns a [`ListOption`](crate::list_option::ListOption) containing
    /// the index of the selection and the owned object selected by the user.
    pub fn raw_prompt(self) -> InquireResult<Vec<ListOption<T>>> {
        let terminal = get_default_terminal()?;
        let mut backend = Backend::new(terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(crate) fn prompt_with_backend<B: MultiSelectBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<Vec<ListOption<T>>> {
        MultiSelectPrompt::new(self)?.prompt(backend)
    }
}
