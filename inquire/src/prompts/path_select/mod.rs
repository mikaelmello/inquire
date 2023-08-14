mod action;
use action::*;
mod path_entry;
pub use path_entry::PathEntry;
mod prompt;
use prompt::*;
mod config;
use config::*;
mod modes;
pub use modes::*;
#[cfg(feature = "crossterm")]
#[cfg(test)]
mod test;

use crate::{
    config::get_configuration,
    error::InquireResult,
    formatter::MultiOptionFormatter,
    list_option::ListOption,
    prompts::prompt::Prompt,
    terminal::get_default_terminal,
    type_aliases::Filter,
    ui::{Backend, MultiSelectBackend, RenderConfig},
    InquireError,
};
use std::path::Path;

/// Prompt for choosing one or multiple files or directories represented by their filesystem paths.  
///
/// The user can:
/// - Navigate at the current level in the file tree pressing ↑ or ↓,
/// - Select or deselect the current path by pressing space,
/// - Navigate higher in the file tree by pressing ← 
/// - Navigate deeper in the file tree by pressing → on a directory
/// - Select all paths by pressing shift + →
/// - Clear selection by pressing shift + ←
/// - Cycle through sorting modes by pressing tab
/// 
/// ### Customizable options:
/// 
/// #### Required:
/// - **Prompt message**: Message shown to the user with the prompt
/// 
/// #### Optional
/// - **Start path**: The path the user begins navigating from.
/// - **Default**: Paths selected when the prompt is first rendered
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
///   - Prints the selected paths string value, joined using a comma as the separator, by default.
/// - **Show hidden**: Whether to show hidden file path entries.
/// - **Show symlinks**: Whether to show symlink path entries.
/// - **Select multiple**: Whether to allow the user to make multiple selections.
/// - **Selection mode**: What files is the user shown and able to select? (See [PathSelectionMode])
/// - **Sorting mode**: How to sort the paths ( See [PathSortingMode] ).
/// - **Page size**: Number of options displayed at once, 7 by default.
/// - **Keep filter flag**: Whether the current filter input should be cleared or not after a selection is made. Defaults to true.
///
/// # Example
///
/// For a full-featured example, check the [GitHub repository](https://github.com/mikaelmello/inquire/blob/main/examples/path_select.rs).
#[derive(Clone)]
pub struct PathSelect<'a, T> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Start path the user will navigate from.
    pub start_path_opt: Option<T>,

    /// Default selected paths  
    pub default: Option<&'a [T]>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Page size of the options displayed to the user.
    pub page_size: usize,

    /// Whether to show hidden files.
    pub show_hidden: bool,

    /// Whether to show symlinks
    pub show_symlinks: bool,

    /// [How to sort](SortingMode) listed files and directories
    pub sorting_mode: PathSortingMode,

    /// Whether to allow selecting multiple files
    pub select_multiple: bool,

    /// The divider to use in the selection list following current-directory entries
    pub divider: &'a str,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: MultiOptionFormatter<'a, PathEntry>,

    /// Whether the current filter typed by the user is kept or cleaned after a selection is made.
    pub keep_filter: bool,

    /// RenderConfig to apply to the rendered interface.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub render_config: RenderConfig<'a>,

    /// The [path selection mode](PathSelectionMode) determines what the user can select.
    pub selection_mode: PathSelectionMode<'a>,
}

impl<'a, T> PathSelect<'a, T>
where
    T: AsRef<Path>,
{
    /// PathEntry formatter used by default in [PathSelect](crate::PathSelect) prompts.
    /// Prints the string value of all selected options, separated by commas.
    ///
    /// See [PathSelect::DEFAULT_PATH_FORMATTER]
    pub const DEFAULT_FORMATTER: MultiOptionFormatter<'a, PathEntry> =
        &|ans| PathSelect::<PathEntry>::DEFAULT_PATH_FORMATTER(ans);

    /// Path formatter used by default in [PathSelect](crate::PathSelect) prompts.
    /// Prints the string value of all selected options, separated by commas.
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// use inquire::list_option::ListOption;
    /// use inquire::{PathSelect, PathEntry};
    /// use std::{fs::FileType, path::PathBuf};
    ///
    /// let formatter = PathSelect::<PathBuf>::DEFAULT_PATH_FORMATTER;
    /// let a = PathBuf::from("/ra/set/nefer.rs");
    /// let mut ans = vec![ListOption::new(0, &a)];
    /// assert_eq!(String::from("/ra/set/nefer.rs"), formatter(ans.as_slice()));
    ///
    /// let b = PathBuf::from("/maat/nut.rs");
    /// ans.push(ListOption::new(3, &b));
    /// assert_eq!(String::from("/ra/set/nefer.rs, /maat/nut.rs"), formatter(ans.as_slice()));
    ///
    /// let c = PathBuf::from("ptah.rs");
    /// ans.push(ListOption::new(7, &c));
    /// assert_eq!(String::from("/ra/set/nefer.rs, /maat/nut.rs, ptah.rs"), formatter(ans.as_slice()));
    /// ```
    pub const DEFAULT_PATH_FORMATTER: MultiOptionFormatter<'a, T> = &|ans| {
        ans.iter()
            .map(|t| PathSelectPrompt::get_path_string(t.value))
            .collect::<Vec<String>>()
            .join(", ")
    };

    /// Default filter function, which only checks if the END component (file name, directory name)
    /// of the path is a match for the filter
    ///
    /// # Examples
    ///
    /// ```
    /// use inquire::PathSelect;
    ///
    /// let filter = PathSelect::<&str>::DEFAULT_FILTER;
    /// assert_eq!(false, filter("fer", &"/nefer/osiris/hotep/ptah/maat",  "/nefer/osiris/hotep/ptah/maat",  0));
    /// assert_eq!(false, filter("tep", &"/nefer/osiris/hotep/ptah/maat",  "/nefer/osiris/hotep/ptah/maat",  1));
    /// assert_eq!(true, filter("aa", &"/nefer/osiris/hotep/ptah/maat",  "/nefer/osiris/hotep/ptah/maat",  2));
    /// assert_eq!(true, filter("ma", &"/nefer/osiris/hotep/ptah/maat",  "/nefer/osiris/hotep/ptah/maat",  3));
    /// assert_eq!(true, filter("ma", &"/nefer/osiris/hotep/ptah/Maat",  "/nefer/osiris/hotep/ptah/Maat",  4));
    /// assert_eq!(true, filter("ma", &"/nefer/osiris/hotep/ptah/Maat.rs",  "/nefer/osiris/hotep/ptah/Maat.rs",  5));
    /// assert_eq!(true, filter("Ma", &"/nefer/osiris/hotep/ptah/maat.rs",  "/nefer/osiris/hotep/ptah/maat.rs",  5));
    /// ```
    pub const DEFAULT_FILTER: Filter<'a, T> = &|filter, as_ref_path, _, _| -> bool {
        let filter = filter.to_lowercase();
        as_ref_path
            .as_ref()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
            .contains(&filter)
    };

    /// Default page size, equal to the global default page size [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_PAGE_SIZE: usize = crate::config::DEFAULT_PAGE_SIZE;

    /// Default value of showing hidden files
    pub const DEFAULT_SHOW_HIDDEN: bool = false;

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> = Some(
        "↑↓ to move, space to select one, \
        → to navigate to path, ← to navigate up, \
        shift+→ to select all, shift+← to clear, \
        tab to change sorting mode",
    );

    /// Default behavior of keeping or cleaning the current filter value.
    pub const DEFAULT_KEEP_FILTER: bool = true;

    /// Default value of showing symlinks
    pub const DEFAULT_SHOW_SYMLINKS: bool = false;

    /// Default value of selecting multiple files
    pub const DEFAULT_SELECT_MULTIPLE: bool = false;

    /// Default visual divider value.
    pub const DEFAULT_DIVIDER: &'a str = "-----";

    /// Creates a [PathSelect] with the provided message and options, along with default configuration values.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            start_path_opt: None,
            default: None,
            divider: Self::DEFAULT_DIVIDER,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            show_hidden: Self::DEFAULT_SHOW_HIDDEN,
            show_symlinks: Self::DEFAULT_SHOW_SYMLINKS,
            select_multiple: Self::DEFAULT_SELECT_MULTIPLE,
            page_size: Self::DEFAULT_PAGE_SIZE,
            formatter: Self::DEFAULT_FORMATTER,
            keep_filter: Self::DEFAULT_KEEP_FILTER,
            render_config: get_configuration(),
            selection_mode: Default::default(),
            sorting_mode: Default::default(),
        }
    }

    /// Sets the keep filter behavior.
    pub fn with_keep_filter(mut self, keep_filter: bool) -> Self {
        self.keep_filter = keep_filter;
        self
    }

    /// Sets the select multiple behavior.
    pub fn with_select_multiple(mut self, select_multiple: bool) -> Self {
        self.select_multiple = select_multiple;
        self
    }

    /// Sets the default sorting mode.
    pub fn with_sorting_mode(mut self, sorting_mode: PathSortingMode) -> Self {
        self.sorting_mode = sorting_mode;
        self
    }

    /// Sets the show hidden behavior.
    pub fn with_show_hidden(mut self, show_hidden: bool) -> Self {
        self.show_hidden = show_hidden;
        self
    }

    /// Sets the show symlinks behavior.
    pub fn with_show_symlinks(mut self, show_symlinks: bool) -> Self {
        self.show_symlinks = show_symlinks;
        self
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

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: MultiOptionFormatter<'a, PathEntry>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the default selected paths.
    pub fn with_default(mut self, default: &'a [T]) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets the divider selected paths.
    pub fn with_divider(mut self, divider: &'a str) -> Self {
        self.divider = divider;
        self
    }

    /// Sets the default starting paths.
    pub fn with_start_path(mut self, start_path: T) -> Self {
        self.start_path_opt = Some(start_path);
        self
    }

    /// Sets the selection mode for picker behavior
    pub fn with_selection_mode(mut self, selection_mode: PathSelectionMode<'a>) -> Self {
        self.selection_mode = selection_mode;
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
    ///>Error::OperationCanceled)`, but `Ok(None)`.
    ///
    /// Meanwhile, if the user does submit an answer, the method wraps the return
    /// type with `Some`.
    pub fn prompt_skippable(self) -> InquireResult<Option<Vec<PathEntry>>> {
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
    pub fn prompt(self) -> InquireResult<Vec<PathEntry>> {
        let terminal = get_default_terminal()?;
        let backend = & mut Backend::new(terminal, self.render_config)?;
        self.prompt_with_backend(backend)
            .map(|op| op.into_iter().map(|o| o.value).collect())
    }

    pub(crate) fn prompt_with_backend<B: MultiSelectBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<Vec<ListOption<PathEntry>>> {
        PathSelectPrompt::new(self)?.prompt(backend)
    }
}
