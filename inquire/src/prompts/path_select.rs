use crate::{
    config,
    formatter::{MultiOptionFormatter, },
    type_aliases::Filter,
    ui::{RenderConfig, MultiSelectBackend, Backend, Key, KeyModifiers},
    validator::{ErrorMessage}, config::get_configuration, error::InquireResult, InquireError, list_option::ListOption, terminal::get_default_terminal, input::Input, utils::paginate,
};
use std::{
    env,
    fmt,
    path::{Component, PathBuf, Path}, collections::{BTreeSet, HashSet}, 
    fs, convert::{TryFrom, TryInto}, 
    ops::Deref,
    ffi::{OsStr, OsString},
};

/// Different path selection modes specify what the user can choose
#[derive(Clone, Default, Eq, PartialEq,)]
pub enum PathSelectionMode<'a> {
    /// The user may pick a file with the given (optional) extension
    File(Option<&'a str>),
    /// The user may pick a directory
    #[default] Directory,
    /// The user may pick multiple paths
    Multiple(Vec<PathSelectionMode<'a>>),
}

/// Path with cached information
#[derive(Clone, Debug, Hash)]
pub struct PathEntry {
    /// The (owned) [path](PathBuf) 
    /// 
    /// Corresponds to the target if this is a symlink  
    pub path: PathBuf,
    /// The [file type](fs::FileType)
    pub file_type: fs::FileType,
    /// The original symlink path
    pub symlink_path_opt: Option<PathBuf>,
}
impl Eq for PathEntry {}
impl PartialEq for PathEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}
impl AsRef<Path> for PathEntry {
    fn as_ref(&self) -> &Path { self.path.as_path() }
}
impl Deref for PathEntry {
    type Target = fs::FileType;
    fn deref(&self) -> &Self::Target { &self.file_type }
}
impl fmt::Display for PathEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.to_string_lossy();
        if let Some(symlink_path) = self.symlink_path_opt.as_ref() { 
            write!(f, "{} -> {path}", symlink_path.to_string_lossy())
        } else { 
            write!(f, "{path}")
        }
    }
}
impl TryFrom<&Path> for PathEntry {
    type Error = InquireError;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let is_symlink = value.is_symlink();
        value.metadata()
            .map_err(Self::Error::from)
            .and_then(|target_metadata| {
                let (path, symlink_path_opt) = if is_symlink {
                    (fs_err::canonicalize(value)?, Some(value.to_path_buf()))
                } else {
                    (value.to_path_buf(), None)
                };                    
                Ok(Self {
                    path,
                    file_type: target_metadata.file_type(),
                    symlink_path_opt
                })
            })
    }
} 
impl TryFrom<fs::DirEntry> for PathEntry {
    type Error = InquireError;
    fn try_from(value: fs::DirEntry) -> Result<Self, Self::Error> {
        Self::try_from(value.path().as_path())
    }
} 
impl TryFrom<fs_err::DirEntry> for PathEntry {
    type Error = InquireError;
    fn try_from(value: fs_err::DirEntry) -> Result<Self, Self::Error> {
        Self::try_from(value.path().as_path())
    }
}
impl PathEntry {
    /// Is this path entry selectable based on the given selection mode?
    pub fn is_selectable<'a>(
        &self, 
        selection_mode: &PathSelectionMode<'a>
    ) -> bool {
        let is_dir = self.is_dir();
        let is_file = self.is_file(); 
        let file_ext_opt = self.path.extension().map(OsStr::to_os_string); 
        match (selection_mode, is_dir, is_file) {
            (PathSelectionMode::Directory, true, _) => true,
            (PathSelectionMode::File(None), _, true) => true,
            (PathSelectionMode::File(Some(extension)), _, true) => {
                file_ext_opt.as_ref()
                    .map(|osstr| {
                        osstr.to_string_lossy().eq_ignore_ascii_case(*extension)
                    }).unwrap_or_default()
            },
            (PathSelectionMode::Multiple(ref path_selection_modes), _, _) => {
                path_selection_modes.iter().any(|submode| {
                    self.is_selectable(submode)
                })
            },
            _ => false
        }
    }

    /// Is this path entry for a symlink?
    pub fn is_symlink(&self) -> bool { self.symlink_path_opt.is_some() }
}
/// Prompt for choosing one or multiple files.
///
/// The user can
#[derive(Clone)]
pub struct PathSelect<'a, T> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Start path shown to the user.
    pub start_path_opt: Option<T>,

    /// Default selected paths  
    pub default: Option<&'a [T]>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Page size of the options displayed to the user.
    pub page_size: usize,

    /// Whether vim mode is enabled. When enabled, the user can
    /// navigate through the options using hjkl.
    pub vim_mode: bool,

    /// Whether to show hidden files.
    pub show_hidden: bool,

    /// Whether to show symlinks
    pub show_symlinks: bool,

    /// The divider to use in the selection list following current-directory entries
    pub divider: &'a str, 

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: MultiOptionFormatter<'a, PathEntry>,

    /// Whether the current filter typed by the user is kept or cleaned after a selection is made.
    pub keep_filter: bool,

    /// RenderConfig to apply to the rendered interface.
    ///
    /// Note: The default render config considers if the NO_COpubLOR environment variable
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
    pub const DEFAULT_FORMATTER: MultiOptionFormatter<'a, PathEntry> = &|ans| {
        PathSelect::<PathEntry>::DEFAULT_PATH_FORMATTER(ans)
    };

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
        .map(|t|  PathSelectPrompt::get_path_string(t.value))
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
        as_ref_path.as_ref().file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
            .contains(&filter)
    };

    /// Default page size, equal to the global default page size [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;

    /// Default value of vim mode, equal to the global default value [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_VIM_MODE: bool = config::DEFAULT_VIM_MODE;

    /// Default value of showing hidden files 
    pub const DEFAULT_SHOW_HIDDEN: bool = false;

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some(r#"↑↓ to move, space to select one, 
        → to navigate to path, ← to navigate up 
        shift + → to select all, shift + ← to clear 
        type to filter"#);

    /// Default behavior of keeping or cleaning the current filter value.
    pub const DEFAULT_KEEP_FILTER: bool = true;

    /// Default value of showing symlinks 
    pub const DEFAULT_SHOW_SYMLINKS: bool = false;

    /// Default visual divider value.
    pub const DEFAULT_DIVIDER: &'a str = "-----";

    /// Creates a [MultiSelect] with the provided message and options, along with default configuration values.
    pub fn new(message: &'a str, start_path_opt: Option<T>) -> Self {
        Self {
            message,
            start_path_opt,
            default: None,
            divider: Self::DEFAULT_DIVIDER,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            show_hidden: Self::DEFAULT_SHOW_HIDDEN,
            show_symlinks: Self::DEFAULT_SHOW_SYMLINKS,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            formatter: Self::DEFAULT_FORMATTER,
            keep_filter: Self::DEFAULT_KEEP_FILTER,
            render_config: get_configuration(),
            selection_mode: Default::default(),
        }
    }

    /// Test if a path is hidden file 
    /// 
    /// ### Problems
    /// This is missing some things described here:
    /// https://en.wikipedia.org/wiki/Hidden_file_and_hidden_directory
    /// - android: .nomedia files that tell smartphone apps not to display/include a folder's contets
    /// - gnome: filenames listed inside a file named ".hidden" in each directory should be hidden
    /// - macos: files with Invisible attribute are usually hidden in Finder but not in `ls`
    /// - windows: files with a Hidden file attribute
    /// - windows: files in folders with a predefined CLSID on the end of their names (Windows Special Folders)
    ///
    /// ```
    /// use inquire::PathSelect;
    /// use std::path::Path;
    ///
    /// assert!(PathSelect::is_path_hidden_file(Path::new("/ra/set/.nut")));
    /// assert!(!PathSelect::is_path_hidden_file(Path::new("/ra/set/nut")));
    /// assert!(PathSelect::is_path_hidden_file(Path::new(".maat")));
    /// assert!(!PathSelect::is_path_hidden_file(Path::new("maat")));
    /// 
    /// ```
    pub fn is_path_hidden_file(t: T) -> bool {
        if cfg!(unix) {
            t.as_ref().file_name().unwrap_or_default().to_str().unwrap_or_default().starts_with(".")
        } else {
            false
        }
    }   

    /// Sets the keep filter behavior.
    pub fn with_keep_filter(mut self, keep_filter: bool) -> Self {
        self.keep_filter = keep_filter;
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

    /// Enables or disables vim_mode.
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
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
    pub fn raw_prompt_skippable(self) -> InquireResult<Option<Vec<ListOption<PathEntry>>>> {
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
    pub fn raw_prompt(self) -> InquireResult<Vec<ListOption<PathEntry>>> {
        let terminal = get_default_terminal()?;
        let mut backend = Backend::new(terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(crate) fn prompt_with_backend<B: MultiSelectBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<Vec<ListOption<PathEntry>>> {
        PathSelectPrompt::new(self)?
            .prompt(backend)
    }
}


struct PathSelectPrompt<'a> {
    message: &'a str,
    options: Vec<PathEntry>,
    divider: &'a str,
    show_symlinks: bool, 
    filtered_options: Vec<usize>,
    data_needs_refresh: bool,
    help_message: Option<&'a str>,
    vim_mode: bool,
    show_hidden: bool,
    cursor_index: usize,
    divider_index: usize,
    selected: HashSet<PathEntry>,
    checked: BTreeSet<usize>,
    page_size: usize,
    input: Input,
    keep_filter: bool,
    formatter: MultiOptionFormatter<'a, PathEntry>,
    error: Option<ErrorMessage>,
    current_path: PathBuf,
    selection_mode: PathSelectionMode<'a>
}

impl<'a> PathSelectPrompt<'a> 
{
    fn new<T: AsRef<Path>>(pso: PathSelect<'a, T>) -> InquireResult<Self> {
        if let Some(default) = pso.default {
            default.iter().try_for_each(|default_item| {
                // Are all of the selected files extant?
                let default_path = default_item.as_ref(); 
                match default_path.try_exists() {
                    Err(err) => {
                        Err(InquireError::InvalidConfiguration(format!(
                            "Checking specified default path (`{default_path:?})` failed with `{err:#?}`"
                        )))
                    },
                    Ok(exists) => {
                        if !exists {
                            Err(InquireError::InvalidConfiguration(format!(
                                "Specified default path `{default_path:?}` does not exist"
                            )))
                        } else {
                            Ok(())
                        }
                    },
                }
            })?;
        }

        let mut start_path = if let Some(start) = pso.start_path_opt {
            start.as_ref().to_path_buf()
        } else {
            env::current_dir()
                .unwrap_or_else(|_| Self::get_root_path_buf())
        };
        if !start_path.is_dir() {
            start_path = start_path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| Self::get_root_path_buf());
        } 

        let selected_options = pso
            .default
            .map_or_else(
                || Result::<_, InquireError>::Ok(HashSet::<PathEntry>::new()),
                |d| d.iter().try_fold(
                    HashSet::new(),
                    |mut s, d| {
                        s.insert(PathEntry::try_from(d.as_ref())?);
                        Ok(s)
                    }
                )
            )?;

        let mut options = Vec::new();
        let mut filtered_options = Vec::new();
        let mut checked = BTreeSet::<usize>::new();
        let show_hidden = pso.show_hidden;
        let show_symlinks = pso.show_symlinks;
        let divider_index = Self::try_update_data_from_selection(
            &start_path,
            &mut options,
            &selected_options,
            &pso.selection_mode,
            &mut checked,
            &mut filtered_options,
            show_hidden,
            show_symlinks,
        )?;

        Ok(Self {
            message: pso.message,
            options,
            filtered_options,
            data_needs_refresh: true,
            help_message: pso.help_message,
            page_size: pso.page_size,
            vim_mode: pso.vim_mode,
            show_symlinks,
            show_hidden,
            keep_filter: pso.keep_filter,
            input: Input::new(),
            error: None, 
            cursor_index: 0,
            checked,
            divider: pso.divider, 
            divider_index,
            formatter: pso.formatter,
            selected: selected_options,
            selection_mode: pso.selection_mode,
            current_path: start_path,
        })
    }

    fn try_update_data_from_selection(
        start_path: &PathBuf,
        options: &mut Vec<PathEntry>,
        selected_options: &HashSet<PathEntry>,
        selection_mode: &PathSelectionMode<'a>,
        checked: &mut BTreeSet<usize>,
        filtered_options: &mut Vec<usize>,
        show_hidden: bool,
        show_symlinks: bool,
    ) -> InquireResult<usize> {
        PathSelectPrompt::try_get_valid_path_options::<&PathBuf>(
            start_path,
            options,
            selection_mode,
            show_hidden,
            show_symlinks,
        )
            .map(|_| {
                Self::update_checked(
                    options,
                    selected_options,
                    checked,
                    filtered_options,
                )       
            })
    }

    fn update_checked(
        options: &mut Vec<PathEntry>,
        selected_options: &HashSet<PathEntry>,
        checked: &mut BTreeSet<usize>,
        filtered_options: &mut Vec<usize>,
    )  -> usize {
        let divider_index = options.len();
        // unlisted selected options are appended
        selected_options.iter().for_each(|selected_entry| {
            if !options.contains(selected_entry) {
                options.push(selected_entry.clone());
            }
        });
        checked.clear();
        checked.extend(selected_options.iter()
            .filter_map(|p| {
                options.iter()
                    .enumerate()
                    .find_map(|(i, o)| (o == p).then(|| i))
            })
        ); 
        filtered_options.clear();
        filtered_options.extend(0..options.len());
                
        divider_index
    }

    fn move_cursor_up(&mut self, qty: usize, wrap: bool) {
        let Self {
            ref mut cursor_index,
            ref filtered_options,
            ..
        } = self;
            if wrap {
                let after_wrap = qty.saturating_sub(*cursor_index);
            *cursor_index = cursor_index
                .checked_sub(qty)
                .unwrap_or_else(|| filtered_options.len().saturating_sub(after_wrap))
            } else {
                *cursor_index = cursor_index.saturating_sub(qty)
            }
        }
        
    fn move_cursor_down(&mut self, qty: usize, wrap: bool) {
        let Self {
            ref mut cursor_index,
            ref filtered_options,
            ..
        } = self; 
        *cursor_index = cursor_index.saturating_add(qty);

        if *cursor_index >= filtered_options.len() {
            *cursor_index = if filtered_options.is_empty() {
                0
            } else if wrap {
                *cursor_index % filtered_options.len()
            } else {
                filtered_options.len().saturating_sub(1)
            };
        }
    }

    fn get_root_path_buf() -> PathBuf {
        <Component as AsRef<Path>>::as_ref(&Component::RootDir).to_path_buf()
    }

    fn get_path_string<T: AsRef<Path>>(p: &T) -> String {
        p.as_ref().to_string_lossy().to_string()
    }

    fn try_get_valid_path_options<T: AsRef<Path>>(
        base_path: T,
        options: &mut Vec<PathEntry>,
        selection_mode: &PathSelectionMode<'a>,
        show_hidden: bool,
        show_symlinks: bool,
    ) -> InquireResult<()> {
        options.clear();
        fs_err::read_dir(base_path.as_ref())
            .map_err(InquireError::from)
            .and_then(|mut read_dir| {
                read_dir.try_for_each(|entry_result| {
                    entry_result
                        .map_err(InquireError::from)
                        .and_then(|dir_entry| {
                            let path_entry = PathEntry::try_from(dir_entry)?;
                            let is_hidden = PathSelect::<&PathBuf>::is_path_hidden_file(&path_entry.path);
                            if path_entry.is_dir() 
                            || path_entry.is_selectable(selection_mode) {
                                if show_hidden || !is_hidden {
                                    if show_symlinks || !path_entry.is_symlink() {
                                        options.push(path_entry);
                                    }
                                } 
                            }
                            Ok(()) 
                        })
                })
            })

    }

    fn render<B: MultiSelectBackend>(&mut self, backend: &mut B) -> InquireResult<()> {
        self.update_path_options()?;
        
        let Self { 
            message, 
            help_message, 
            options, 
            filtered_options,
            cursor_index, 
            page_size, 
            input, 
            error, 
            // selected,
            checked,
            .. 
        } = self; 

        

        let prompt = message;
        backend.frame_setup()?;
        
        if let Some(error_message) = error {
            backend.render_error_message(error_message)?;
        }

        backend.render_multiselect_prompt(prompt, input)?;

        
        
        let choices = filtered_options
            .iter()
            // .chain(selected.iter())
            .cloned()
            .map(|i| {
                ListOption::new(
                    i,
                    options.get(i).expect("must get path entry")
                        .clone()
                )
            })
            .collect::<Vec<ListOption<PathEntry>>>();

        let page = paginate(*page_size, &choices, Some(*cursor_index));
        backend.render_options(page, checked)?;
        
        if let Some(help) = help_message {
            backend.render_help_message(help)?;
        }

        backend.frame_finish()?;
        
        Ok(())
    }

    fn update_path_options(&mut self) -> InquireResult<()> {
        if self.data_needs_refresh {
            Self::try_update_data_from_selection(
                &self.current_path,
                &mut self.options,
                &self.selected,
                &self.selection_mode,
                &mut self.checked,
                &mut self.filtered_options,
                self.show_hidden,
                self.show_symlinks,
            )?;
            self.data_needs_refresh = false;
        }
        
        Ok(())
        
    }

    fn toggle_cursor_selection(&mut self) {
        let Self { 
            ref filtered_options, 
            ref options, 
            ref cursor_index, 
            ref selection_mode,
            ref keep_filter, 
            ref mut checked, 
            ref mut input,
            ref mut selected,
            ..
        } = self; 
        
        let option_index = match filtered_options.get(*cursor_index) {
            Some(val) => val,
            None => return,
        };
        let option_entry = options.get(*option_index)
            .expect("must get option_entry");

        if option_entry.is_selectable(selection_mode) {

            if checked.contains(option_index) {
                checked.remove(option_index);
                selected.remove(option_entry);
            } else {
                checked.insert(*option_index);
                selected.insert(option_entry.clone());
            }
            
            if !*keep_filter {
                input.clear();
            }
        }


    }   

    fn on_change(&mut self, key: Key) {
        let Self {
            ref vim_mode, 
            ref page_size, 
            ref keep_filter,
            ref options, 
            ref selection_mode,
            ref mut cursor_index, 
            ref mut checked,
            ref mut input, 
            ref mut selected,
            ref mut filtered_options,
            ref mut current_path,
            ref mut data_needs_refresh,
            ..
        } = self;
        match key {
            Key::Up(KeyModifiers::NONE) => {
                self.move_cursor_up(1, true)
            },
            Key::Char('k', KeyModifiers::NONE) if *vim_mode => {
                self.move_cursor_up(1, true)
            },
            Key::PageUp => self.move_cursor_up(*page_size, true),
            Key::Home => self.move_cursor_up(usize::MAX, true),
            
            Key::Down(KeyModifiers::NONE) => {
                self.move_cursor_down(1, true)
            },
            Key::Char('j', KeyModifiers::NONE) if *vim_mode => {
                self.move_cursor_down(1, true)
            },
            Key::PageDown => self.move_cursor_down(*page_size, true),
            Key::End => self.move_cursor_down(usize::MAX, true),

            Key::Char(' ', KeyModifiers::NONE) => self.toggle_cursor_selection(),
            Key::Right(KeyModifiers::NONE) => {
                match filtered_options.get(*cursor_index) {
                    Some(option_index) => {
                        match options.get(*option_index) {
                            Some(PathEntry{file_type, path, ..}) if file_type.is_dir() => {
                                *current_path = path.to_path_buf();
                                *data_needs_refresh = true;
                            },
                            _ => {}
                        }
                    },
                    _ => { /* It might be an empty directory*/}
                }
            },
            Key::Right(KeyModifiers::SHIFT) => {
                checked.clear();
                filtered_options.iter().for_each(|idx|{
                    let option_entry = options.get(*idx)
                        .expect("must get selected path");
                    if option_entry.is_selectable(selection_mode) {
                        checked.insert(*idx);
                        selected.insert(option_entry.clone()); 
                    }
                });

                if !*keep_filter {
                    input.clear();
                }
            },
            Key::Left(KeyModifiers::SHIFT) => {
                checked.clear();
                selected.clear();
                if !*keep_filter {
                    input.clear();
                }
            },
            Key::Left(KeyModifiers::NONE) => {
                if let Some(parent) = current_path.parent() {
                    *current_path = parent.to_path_buf();
                    *data_needs_refresh = true;
                } 
            },
            key => {
                let dirty = input.handle_key(key);
                if dirty {
                    let options = Self::filter_options(options, input);
                    if options.len() <= *cursor_index {
                        *cursor_index = options.len().saturating_sub(1);
                    }
                    *filtered_options = options;
                }
            } 
        }
    }

    fn filter_options(
        options: &[PathEntry],
        input: &Input
    ) -> Vec<usize> {
        options.iter()
            .enumerate()
            .filter_map(|(i, path_entry)| {
                match input.content() {
                    val if val.is_empty() => Some(i),
                    val if PathSelect::<PathEntry>::DEFAULT_FILTER(
                        val, path_entry, "", i
                    ) => Some(i),
                    _ => None
                }
            })
            .collect()
    }

    fn get_final_answer(&mut self) -> Vec<ListOption<PathEntry>> {
        let Self {
            ref mut options,
            ref checked,
            ..
        } = self;
        let mut answer = Vec::with_capacity(checked.len());
        answer.extend(checked.iter().rev().cloned()
            .map(|index| {
                let value = options.swap_remove(index);
                ListOption{ value, index }
            })
        );
        answer.reverse();
        answer
    }

    fn prompt<B: MultiSelectBackend>(
        mut self,
        backend: &mut B 
    ) -> InquireResult<Vec<ListOption<PathEntry>>> {
        'render_listen: loop {
            self.render(backend)?;
    
            let key = backend.read_key()?;

            match key {
                Key::Interrupt => interrupt_prompt!(),
                Key::Cancel => cancel_prompt!(backend, self.message),
                Key::Submit => break 'render_listen,
                key => self.on_change(key),
            } 
        }

        let final_answer = self.get_final_answer();
        let refs = final_answer.iter()
            .map(ListOption::as_ref)
            .collect::<Vec<ListOption<_>>>();
        let formatted = (self.formatter)(refs.as_slice());
        finish_prompt_with_answer!(backend, self.message, &formatted, final_answer);
    }
}