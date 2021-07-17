use std::iter::FromIterator;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    answer::OptionAnswer,
    config::{self, Filter},
    error::{InquireError, InquireResult},
    formatter::{self, OptionFormatter},
    key::{Key, KeyModifiers},
    renderer::Renderer,
    terminal::Terminal,
    utils::paginate,
};

/// Presents a message to the user and a list of options for the user to choose from.
/// The user is able to choose only one option.
#[derive(Copy, Clone)]
pub struct Select<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Options displayed to the user.
    pub options: &'a [&'a str],

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
    pub filter: Filter<'a>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: OptionFormatter,
}

impl<'a> Select<'a> {
    /// Default formatter.
    pub const DEFAULT_FORMATTER: OptionFormatter = formatter::DEFAULT_OPTION_FORMATTER;
    /// Default filter.
    pub const DEFAULT_FILTER: Filter<'a> = config::DEFAULT_FILTER;
    /// Default page size.
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;
    /// Default value of vim mode.
    pub const DEFAULT_VIM_MODE: bool = config::DEFAULT_VIM_MODE;
    /// Default starting cursor index.
    pub const DEFAULT_STARTING_CURSOR: usize = 0;
    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("↑↓ to move, space or enter to select, type to filter");

    /// Creates a [Select] with the provided message and options, along with default configuration values.
    pub fn new(message: &'a str, options: &'a [&str]) -> Self {
        Self {
            message,
            options,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            starting_cursor: Self::DEFAULT_STARTING_CURSOR,
            filter: Self::DEFAULT_FILTER,
            formatter: Self::DEFAULT_FORMATTER,
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
    pub fn with_filter(mut self, filter: Filter<'a>) -> Self {
        self.filter = filter;
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: OptionFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the starting cursor index.
    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Self {
        self.starting_cursor = starting_cursor;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to them.
    pub fn prompt(self) -> InquireResult<OptionAnswer> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(
        self,
        renderer: &mut Renderer,
    ) -> InquireResult<OptionAnswer> {
        SelectPrompt::new(self)?.prompt(renderer)
    }
}

struct SelectPrompt<'a> {
    message: &'a str,
    options: &'a [&'a str],
    help_message: Option<&'a str>,
    vim_mode: bool,
    cursor_index: usize,
    page_size: usize,
    filter_value: Option<String>,
    filtered_options: Vec<usize>,
    filter: Filter<'a>,
    formatter: OptionFormatter,
}

impl<'a> SelectPrompt<'a> {
    fn new(so: Select<'a>) -> InquireResult<Self> {
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

        Ok(Self {
            message: so.message,
            options: so.options,
            help_message: so.help_message,
            vim_mode: so.vim_mode,
            cursor_index: so.starting_cursor,
            page_size: so.page_size,
            filter_value: None,
            filtered_options: Vec::from_iter(0..so.options.len()),
            filter: so.filter,
            formatter: so.formatter,
        })
    }

    fn filter_options(&self) -> Vec<usize> {
        self.options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| match &self.filter_value {
                Some(val) if (self.filter)(&val, opt, i) => Some(i),
                Some(_) => None,
                None => Some(i),
            })
            .collect()
    }

    fn move_cursor_up(&mut self) {
        self.cursor_index = self
            .cursor_index
            .checked_sub(1)
            .or(self.filtered_options.len().checked_sub(1))
            .unwrap_or_else(|| 0);
    }

    fn move_cursor_down(&mut self) {
        self.cursor_index = self.cursor_index.saturating_add(1);
        if self.cursor_index >= self.filtered_options.len() {
            self.cursor_index = 0;
        }
    }

    fn on_change(&mut self, key: Key) {
        let old_filter = self.filter_value.clone();

        match key {
            Key::Up(KeyModifiers::NONE) => self.move_cursor_up(),
            Key::Char('k', KeyModifiers::NONE) if self.vim_mode => self.move_cursor_up(),
            Key::Tab | Key::Down(KeyModifiers::NONE) => self.move_cursor_down(),
            Key::Char('j', KeyModifiers::NONE) if self.vim_mode => self.move_cursor_down(),
            Key::Backspace => {
                if let Some(filter) = &self.filter_value {
                    let len = filter[..].graphemes(true).count();
                    let new_len = len.saturating_sub(1);
                    self.filter_value = Some(filter[..].graphemes(true).take(new_len).collect());
                }
            }
            Key::Char(c, KeyModifiers::NONE) => match &mut self.filter_value {
                Some(val) => val.push(c),
                None => self.filter_value = Some(String::from(c)),
            },
            _ => {}
        }

        if self.filter_value != old_filter {
            let options = self.filter_options();
            if options.len() > 0 && options.len() <= self.cursor_index {
                self.cursor_index = options.len().saturating_sub(1);
            }
            self.filtered_options = options;
        }
    }

    fn get_final_answer(&self) -> InquireResult<OptionAnswer> {
        self.filtered_options
            .get(self.cursor_index)
            .and_then(|i| self.options.get(*i).map(|opt| OptionAnswer::new(*i, opt)))
            .ok_or(InquireError::InvalidState(format!(
                "Index {} is not in the range of options with len {}",
                self.cursor_index,
                self.options.len()
            )))
    }

    fn render(&mut self, renderer: &mut Renderer) -> InquireResult<()> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        renderer.print_prompt(&prompt, None, self.filter_value.as_deref())?;

        let choices = self
            .filtered_options
            .iter()
            .cloned()
            .map(|i| OptionAnswer::new(i, self.options.get(i).unwrap()))
            .collect::<Vec<OptionAnswer>>();

        let (paginated_opts, rel_sel) = paginate(self.page_size, &choices, self.cursor_index);

        for (idx, opt) in paginated_opts.iter().enumerate() {
            renderer.print_option(rel_sel == idx, &opt.value)?;
        }

        if let Some(help_message) = self.help_message {
            renderer.print_help(help_message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> InquireResult<OptionAnswer> {
        let final_answer: OptionAnswer;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Cancel => return Err(InquireError::OperationCanceled),
                Key::Submit | Key::Char(' ', KeyModifiers::NONE) => match self.get_final_answer() {
                    Ok(answer) => {
                        final_answer = answer;
                        break;
                    }
                    Err(err) => return Err(err),
                },
                key => self.on_change(key),
            }
        }

        let formatted = (self.formatter)(&final_answer);

        renderer.cleanup(&self.message, &formatted)?;

        Ok(final_answer)
    }
}
