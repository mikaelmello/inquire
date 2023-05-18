use crate::{
    error::InquireResult,
    formatter::MultiOptionFormatter,
    input::{Input, InputActionResult},
    list_option::ListOption,
    prompts::{
        path_select::{
            PathEntry, PathSelect, PathSelectConfig, PathSelectPromptAction, PathSelectionMode,
        },
        prompt::{ActionResult, Prompt},
    },
    ui::MultiSelectBackend,
    utils::paginate,
    validator::ErrorMessage,
    InquireError, SortingMode,
};
use std::{
    collections::{BTreeSet, HashSet},
    convert::TryFrom,
    env,
    path::{Component, Path, PathBuf},
};

pub struct PathSelectPrompt<'a> {
    message: &'a str,
    options: Vec<PathEntry>,
    #[allow(dead_code)]
    divider: &'a str,
    show_symlinks: bool,
    select_multiple: bool,
    sorting_mode: SortingMode,
    filtered_options: Vec<usize>,
    data_needs_refresh: bool,
    help_message: Option<&'a str>,
    config: PathSelectConfig,
    show_hidden: bool,
    cursor_index: usize,
    #[allow(dead_code)]
    divider_index: usize,
    selected: HashSet<PathEntry>,
    checked: BTreeSet<usize>,
    input: Input,
    formatter: MultiOptionFormatter<'a, PathEntry>,
    error: Option<ErrorMessage>,
    current_path: PathBuf,
    selection_mode: PathSelectionMode<'a>,
}

impl<'a> PathSelectPrompt<'a> {
    pub fn new<T: AsRef<Path>>(pso: PathSelect<'a, T>) -> InquireResult<Self> {
        let config = PathSelectConfig::from(&pso);

        if let Some(default) = pso.default {
            default.iter().try_for_each(|default_item| {
                // Are all of the selected files extant?
                let default_path = default_item.as_ref();
                if !default_path.exists() {
                    Err(InquireError::InvalidConfiguration(format!(
                        "Specified default path `{default_path:?}` does not exist"
                    )))
                } else {
                    Ok(())
                }
            })?;
        }

        let mut start_path = if let Some(start) = pso.start_path_opt {
            start.as_ref().to_path_buf()
        } else {
            env::current_dir().unwrap_or_else(|_| Self::get_root_path_buf())
        };
        if !start_path.is_dir() {
            start_path = start_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| Self::get_root_path_buf());
        }

        let selected_options = pso.default.map_or_else(
            || Result::<_, InquireError>::Ok(HashSet::<PathEntry>::new()),
            |d| {
                d.iter().try_fold(HashSet::new(), |mut s, d| {
                    s.insert(PathEntry::try_from(d.as_ref())?);
                    Ok(s)
                })
            },
        )?;

        let mut options = Vec::new();
        let mut filtered_options = Vec::new();
        let mut checked = BTreeSet::<usize>::new();
        let show_hidden = pso.show_hidden;
        let show_symlinks = pso.show_symlinks;
        let sorting_mode = pso.sorting_mode;
        let divider_index = Self::try_update_data_from_selection(
            &start_path,
            &mut options,
            &selected_options,
            &pso.selection_mode,
            &mut checked,
            &mut filtered_options,
            show_hidden,
            show_symlinks,
            sorting_mode,
        )?;
        Ok(Self {
            message: pso.message,
            options,
            filtered_options,
            data_needs_refresh: true,
            help_message: pso.help_message,
            show_symlinks,
            show_hidden,
            select_multiple: pso.select_multiple,
            sorting_mode,
            config,
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
        sorting_mode: SortingMode,
    ) -> InquireResult<usize> {
        PathSelectPrompt::try_get_valid_path_options::<&PathBuf>(
            start_path,
            options,
            selection_mode,
            show_hidden,
            show_symlinks,
            sorting_mode,
        )
        .map(|_| Self::update_checked(options, selected_options, checked, filtered_options))
    }

    fn update_checked(
        options: &mut Vec<PathEntry>,
        selected_options: &HashSet<PathEntry>,
        checked: &mut BTreeSet<usize>,
        filtered_options: &mut Vec<usize>,
    ) -> usize {
        let divider_index = options.len();
        // unlisted selected options are appended
        selected_options.iter().for_each(|selected_entry| {
            if !options.contains(selected_entry) {
                options.push(selected_entry.clone());
            }
        });
        checked.clear();
        checked.extend(selected_options.iter().filter_map(|p| {
            options
                .iter()
                .enumerate()
                .find_map(|(i, o)| (o == p).then(|| i))
        }));
        filtered_options.clear();
        filtered_options.extend(0..options.len());

        divider_index
    }

    fn select(
        option_entry: &PathEntry,
        option_index: usize,
        checked: &mut BTreeSet<usize>,
        selected: &mut HashSet<PathEntry>,
        select_multiple: bool,
    ) {
        if !select_multiple {
            checked.clear();
            selected.clear();
        }
        checked.insert(option_index);
        selected.insert(option_entry.clone());
    }

    fn move_cursor_up(&mut self, qty: usize, wrap: bool) -> ActionResult {
        let Self {
            ref cursor_index,
            ref filtered_options,
            ..
        } = self;
        let new_position = if wrap {
            let after_wrap = qty.saturating_sub(*cursor_index);
            cursor_index
                .checked_sub(qty)
                .unwrap_or_else(|| filtered_options.len().saturating_sub(after_wrap))
        } else {
            cursor_index.saturating_sub(qty)
        };

        self.update_cursor_position(new_position)
    }

    fn move_cursor_down(&mut self, qty: usize, wrap: bool) -> ActionResult {
        let Self {
            ref cursor_index,
            ref filtered_options,
            ..
        } = self;
        let mut new_position = cursor_index.saturating_add(qty);

        if new_position >= filtered_options.len() {
            new_position = if filtered_options.is_empty() {
                0
            } else if wrap {
                new_position % filtered_options.len()
            } else {
                filtered_options.len().saturating_sub(1)
            };
        }

        self.update_cursor_position(new_position)
    }

    fn update_cursor_position(&mut self, new_position: usize) -> ActionResult {
        let Self {
            ref mut cursor_index,
            ..
        } = self;
        if new_position != *cursor_index {
            *cursor_index = new_position;
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    fn get_root_path_buf() -> PathBuf {
        <Component as AsRef<Path>>::as_ref(&Component::RootDir).to_path_buf()
    }

    pub fn get_path_string<T: AsRef<Path>>(p: &T) -> String {
        p.as_ref().to_string_lossy().to_string()
    }

    fn try_get_valid_path_options<T: AsRef<Path>>(
        base_path: T,
        options: &mut Vec<PathEntry>,
        selection_mode: &PathSelectionMode<'a>,
        show_hidden: bool,
        show_symlinks: bool,
        sorting_mode: SortingMode
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
                            let is_hidden =
                                PathSelect::<&PathBuf>::is_path_hidden_file(&path_entry.path);
                            if path_entry.is_dir() || path_entry.is_selectable(selection_mode) {
                                if show_hidden || !is_hidden {
                                    if show_symlinks || !path_entry.is_symlink() {
                                        options.push(path_entry);
                                    }
                                }
                            }
                            Ok(())
                        })
                })
            })?;
        options.sort_by(|a, b| PathEntry::sort_by_mode(a, b, sorting_mode));
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
                self.sorting_mode,
            )?;
            self.data_needs_refresh = false;
        }

        Ok(())
    }

    fn toggle_cursor_selection(&mut self) -> ActionResult {
        let Self {
            ref filtered_options,
            ref options,
            ref cursor_index,
            ref selection_mode,
            ref select_multiple,
            config: PathSelectConfig {
                ref keep_filter, ..
            },
            ref mut checked,
            ref mut input,
            ref mut selected,
            ..
        } = self;

        let option_index = match filtered_options.get(*cursor_index) {
            Some(val) => val,
            None => return ActionResult::Clean,
        };
        let option_entry = options.get(*option_index).expect("must get option_entry");

        if option_entry.is_selectable(selection_mode) {
            if checked.contains(option_index) {
                checked.remove(option_index);
                selected.remove(option_entry);
            } else {
                Self::select(
                    option_entry,
                    *option_index,
                    checked,
                    selected,
                    *select_multiple,
                )
            }

            if !*keep_filter {
                input.clear();
            }
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    fn filter_options(options: &[PathEntry], input: &Input) -> Vec<usize> {
        options
            .iter()
            .enumerate()
            .filter_map(|(i, path_entry)| match input.content() {
                val if val.is_empty() => Some(i),
                val if PathSelect::<PathEntry>::DEFAULT_FILTER(val, path_entry, "", i) => Some(i),
                _ => None,
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
        answer.extend(checked.iter().rev().cloned().map(|index| {
            let value = options.swap_remove(index);
            ListOption { value, index }
        }));
        answer.reverse();
        answer
    }
}

impl<'a, B> Prompt<B, PathSelectConfig, PathSelectPromptAction, Vec<ListOption<PathEntry>>>
    for PathSelectPrompt<'a>
where
    B: MultiSelectBackend,
{
    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &PathSelectConfig {
        &self.config
    }

    fn format_answer(&self, answer: &Vec<ListOption<PathEntry>>) -> String {
        let refs = answer
            .iter()
            .map(ListOption::as_ref)
            .collect::<Vec<ListOption<_>>>();
        (self.formatter)(refs.as_slice())
    }

    fn submit(&mut self) -> InquireResult<Option<Vec<ListOption<PathEntry>>>> {
        Ok(Some(self.get_final_answer()))
    }

    fn render(&self, backend: &mut B) -> InquireResult<()> {
        let Self {
            message,
            help_message,
            options,
            filtered_options,
            cursor_index,
            config: PathSelectConfig { ref page_size, .. },
            input,
            error,
            // selected,
            checked,
            ..
        } = self;

        let prompt = message;

        if let Some(error_message) = error {
            backend.render_error_message(error_message)?;
        }

        backend.render_multiselect_prompt(prompt, input)?;

        let choices = filtered_options
            .iter()
            // .chain(selected.iter())
            .cloned()
            .map(|i| ListOption::new(i, options.get(i).expect("must get path entry").clone()))
            .collect::<Vec<ListOption<PathEntry>>>();

        let page = paginate(*page_size, &choices, Some(*cursor_index));
        backend.render_options(page, checked)?;

        if let Some(help) = help_message {
            backend.render_help_message(help)?;
        }
        Ok(())
    }

    fn handle(
        &mut self,
        action: PathSelectPromptAction,
    ) -> InquireResult<crate::prompts::prompt::ActionResult> {
        let Self {
            config:
                PathSelectConfig {
                    ref page_size,
                    ref keep_filter,
                    ..
                },
            ref options,
            ref select_multiple,
            ref selection_mode,
            ref mut cursor_index,
            ref mut checked,
            ref mut input,
            ref mut selected,
            ref mut filtered_options,
            ref mut current_path,
            ref mut data_needs_refresh,
            ref mut sorting_mode,
            ..
        } = self;
        let result = match action {
            PathSelectPromptAction::MoveUp => self.move_cursor_up(1, true),
            PathSelectPromptAction::PageUp => self.move_cursor_up(*page_size, true),
            PathSelectPromptAction::MoveToStart => self.move_cursor_up(usize::MAX, true),

            PathSelectPromptAction::MoveDown => self.move_cursor_down(1, true),
            PathSelectPromptAction::PageDown => self.move_cursor_down(*page_size, true),
            PathSelectPromptAction::MoveToEnd => self.move_cursor_down(usize::MAX, true),
            PathSelectPromptAction::ChangeSortMode => {
                *sorting_mode = (*sorting_mode).next();
                *data_needs_refresh = true;
                ActionResult::NeedsRedraw 
            }
            PathSelectPromptAction::ToggleCurrentOption => self.toggle_cursor_selection(),
            PathSelectPromptAction::NavigateDeeper => {
                match filtered_options.get(*cursor_index) {
                    Some(option_index) => match options.get(*option_index) {
                        Some(PathEntry {
                            file_type, path, ..
                        }) if file_type.is_dir() => {
                            *current_path = path.to_path_buf();
                            *data_needs_refresh = true;
                            *cursor_index = 0;
                            ActionResult::NeedsRedraw
                        }
                        _ => ActionResult::Clean,
                    },
                    _ => {
                        /* It might be an empty directory*/
                        ActionResult::Clean
                    }
                }
            }

            PathSelectPromptAction::NavigateHigher => {
                if let Some(parent) = current_path.parent() {
                    *current_path = parent.to_path_buf();
                    *data_needs_refresh = true;
                    *cursor_index = 0;
                }
                ActionResult::NeedsRedraw
            }
            PathSelectPromptAction::SelectAll => {
                checked.clear();
                filtered_options.iter().for_each(|option_index| {
                    let option_entry = options.get(*option_index).expect("must get selected path");
                    if option_entry.is_selectable(selection_mode)
                        && (*select_multiple || option_index == cursor_index)
                    {
                        Self::select(
                            option_entry,
                            *option_index,
                            checked,
                            selected,
                            *select_multiple,
                        );
                    }
                });

                if !*keep_filter {
                    input.clear();
                }
                ActionResult::NeedsRedraw
            }
            PathSelectPromptAction::ClearSelections => {
                checked.clear();
                selected.clear();
                if !*keep_filter {
                    input.clear();
                }
                ActionResult::NeedsRedraw
            }
            PathSelectPromptAction::FilterInput(input_action) => {
                let input_action_result = input.handle(input_action);
                if let InputActionResult::ContentChanged = input_action_result {
                    let options = Self::filter_options(options, input);
                    if options.len() <= *cursor_index {
                        *cursor_index = options.len().saturating_sub(1);
                    }
                    *filtered_options = options;
                }
                input_action_result.into()
            }
        };
        if result == ActionResult::NeedsRedraw {
            self.update_path_options()?;
        }
        Ok(result)
    }
}
