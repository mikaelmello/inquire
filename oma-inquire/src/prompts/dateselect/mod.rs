mod action;
mod config;
mod prompt;
#[cfg(test)]
#[cfg(feature = "crossterm")]
mod test;

pub use action::*;

use chrono::NaiveDate;

use crate::{
    config::get_configuration,
    date_utils::get_current_date,
    error::{InquireError, InquireResult},
    formatter::{self, DateFormatter},
    prompts::prompt::Prompt,
    terminal::get_default_terminal,
    ui::{date::DateSelectBackend, Backend, RenderConfig},
    validator::DateValidator,
};

use self::prompt::DateSelectPrompt;

/// Prompt that allows user to select a date (time not supported) from an interactive calendar. Available via the `date` feature.
///
/// By default, the initial selected date is the current date. The user can navigate through the calendar by pressing the keyboard arrows. If the user also presses the control key along with the arrows, the user will be able to "fast-forward" to previous or next months or years.
///
/// More specifically:
/// - Left arrow moves to the day previous to the one selected, and to the month previous to the one selected when pressed with `ctrl`.
/// - Analogously, right arrow does the same, but moving to the next day or month.
/// - Up arrow moves to the day above to the one selected, basically a week before the selected date. When pressed with `ctrl`, it moves to the previous year.
/// - Analogously, the down arrow moves to a week later or a year later.
///
/// Finally, the user selects a date by pressing the space or enter keys.
///
/// `DateSelect` prompts provide several options of configuration:
///
/// - **Prompt message**: Required when creating the prompt.
/// - **Default value**: Default value selected when the calendar is displayed and the one select if the user submits without any previous actions. Current date by default.
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
///   - Formats to "Month Day, Year" by default.
/// - **Validators**: Custom validators to the user's selected date, displaying an error message if the date does not pass the requirements.
/// - **Week start**: Which day of the week should be displayed in the first column of the calendar, Sunday by default.
/// - **Min and max date**: Inclusive boundaries of allowed dates in the interactive calendar. If any boundary is set, the user will not be able to move past them, consequently not being able to select any dates out of the allowed range.
///
/// # Example
///
/// ```no_run
/// use chrono::{NaiveDate, Weekday};
/// use inquire::DateSelect;
///
/// let date = DateSelect::new("When do you want to travel?")
///     .with_starting_date(NaiveDate::from_ymd(2021, 8, 1))
///     .with_min_date(NaiveDate::from_ymd(2021, 8, 1))
///     .with_max_date(NaiveDate::from_ymd(2021, 12, 31))
///     .with_week_start(Weekday::Mon)
///     .with_help_message("Possible flights will be displayed according to the selected date")
///     .prompt();
///
/// match date {
///     Ok(_) => println!("No flights available for this date."),
///     Err(_) => println!("There was an error in the system."),
/// }
/// ```
#[derive(Clone)]
pub struct DateSelect<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// First day of the week when displaying week rows.
    pub week_start: chrono::Weekday,

    /// Starting date to be selected.
    pub starting_date: NaiveDate,

    /// Min date allowed to be selected.
    pub min_date: Option<NaiveDate>,

    /// Max date allowed to be selected.
    pub max_date: Option<NaiveDate>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: DateFormatter<'a>,

    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Vec<Box<dyn DateValidator>>,

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

impl<'a> DateSelect<'a> {
    /// Default formatter, set to [DEFAULT_DATE_FORMATTER](crate::formatter::DEFAULT_DATE_FORMATTER)
    pub const DEFAULT_FORMATTER: DateFormatter<'a> = formatter::DEFAULT_DATE_FORMATTER;

    /// Default value of vim mode. It is true because there is no typing functionality to be lost here.
    pub const DEFAULT_VIM_MODE: bool = true;

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("arrows to move, []{} move months and years, enter to select");

    /// Default validators added to the [DateSelect] prompt, none.
    pub const DEFAULT_VALIDATORS: Vec<Box<dyn DateValidator>> = vec![];

    /// Default week start.
    pub const DEFAULT_WEEK_START: chrono::Weekday = chrono::Weekday::Sun;

    /// Default min date.
    pub const DEFAULT_MIN_DATE: Option<NaiveDate> = None;

    /// Default max date.
    pub const DEFAULT_MAX_DATE: Option<NaiveDate> = None;

    /// Creates a [DateSelect] with the provided message, along with default configuration values.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            starting_date: get_current_date(),
            min_date: Self::DEFAULT_MIN_DATE,
            max_date: Self::DEFAULT_MAX_DATE,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            formatter: Self::DEFAULT_FORMATTER,
            validators: Self::DEFAULT_VALIDATORS,
            week_start: Self::DEFAULT_WEEK_START,
            render_config: get_configuration(),
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

    /// Sets the default date of the prompt. Equivalent to [DateSelect::with_starting_date](DateSelect::with_starting_date).
    pub fn with_default(self, default: NaiveDate) -> Self {
        self.with_starting_date(default)
    }

    /// Sets the week start.
    pub fn with_week_start(mut self, week_start: chrono::Weekday) -> Self {
        self.week_start = week_start;
        self
    }

    /// Sets the min date.
    pub fn with_min_date(mut self, min_date: NaiveDate) -> Self {
        self.min_date = Some(min_date);
        self
    }

    /// Sets the max date.
    pub fn with_max_date(mut self, max_date: NaiveDate) -> Self {
        self.max_date = Some(max_date);
        self
    }

    /// Sets the starting date. Equivalent to [DateSelect::with_default](DateSelect::with_default).
    pub fn with_starting_date(mut self, starting_date: NaiveDate) -> Self {
        self.starting_date = starting_date;
        self
    }

    /// Adds a validator to the collection of validators. You might want to use this feature
    /// in case you need to limit the user to specific choices, such as not allowing weekends.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validator<V>(mut self, validator: V) -> Self
    where
        V: DateValidator + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }

    /// Adds the validators to the collection of validators in the order they are given.
    /// You might want to use this feature in case you need to limit the user to specific
    /// choices, such as not allowing weekends.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validators(mut self, validators: &[Box<dyn DateValidator>]) -> Self {
        for validator in validators {
            self.validators.push(validator.clone());
        }
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: DateFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the provided color theme to this prompt.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still support NO_COLOR, you will have to do this on your end.
    pub fn with_render_config(mut self, render_config: RenderConfig<'a>) -> Self {
        self.render_config = render_config;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    ///
    /// This method is intended for flows where the user skipping/cancelling
    /// the prompt - by pressing ESC - is considered normal behavior. In this case,
    /// it does not return `Err(InquireError::OperationCanceled)`, but `Ok(None)`.
    ///
    /// Meanwhile, if the user does submit an answer, the method wraps the return
    /// type with `Some`.
    pub fn prompt_skippable(self) -> InquireResult<Option<NaiveDate>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<NaiveDate> {
        let (input_reader, terminal) = get_default_terminal()?;
        let mut backend = Backend::new(input_reader, terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(crate) fn prompt_with_backend<B: DateSelectBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<NaiveDate> {
        DateSelectPrompt::new(self)?.prompt(backend)
    }
}
