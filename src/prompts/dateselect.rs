use chrono::{Datelike, Duration, NaiveDate};
use std::{
    cmp::{max, min},
    ops::Add,
};

use crate::{
    config::get_configuration,
    date_utils::{get_current_date, get_month},
    error::{InquireError, InquireResult},
    formatter::{self, DateFormatter},
    terminal::{get_default_terminal, Terminal},
    ui::{date::DateSelectBackend, Backend, Key, KeyModifiers, RenderConfig},
    validator::{DateValidator, ErrorMessage, Validation},
};

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

    /// Whether vim mode is enabled. When enabled, the user can
    /// navigate through the options using hjkl.
    pub vim_mode: bool,

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
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub render_config: RenderConfig,
}

impl<'a> DateSelect<'a> {
    /// Default formatter, set to [DEFAULT_DATE_FORMATTER](crate::formatter::DEFAULT_DATE_FORMATTER)
    pub const DEFAULT_FORMATTER: DateFormatter<'a> = formatter::DEFAULT_DATE_FORMATTER;

    /// Default value of vim mode. It is true because there is no typing functionality to be lost here.
    pub const DEFAULT_VIM_MODE: bool = true;

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("arrows to move, with ctrl to move months and years, enter to select");

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
            vim_mode: Self::DEFAULT_VIM_MODE,
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
        // Directly make space for at least 5 elements, so we won't to re-allocate too often when
        // calling this function repeatedly.
        if self.validators.capacity() == 0 {
            self.validators.reserve(5);
        }

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

    /// Enables or disables vim_mode.
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
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
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub fn with_render_config(mut self, render_config: RenderConfig) -> Self {
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
        let terminal = get_default_terminal()?;
        let mut backend = Backend::new(terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(crate) fn prompt_with_backend<T: Terminal>(
        self,
        backend: &mut Backend<T>,
    ) -> InquireResult<NaiveDate> {
        DateSelectPrompt::new(self)?.prompt(backend)
    }
}

struct DateSelectPrompt<'a> {
    message: &'a str,
    current_date: NaiveDate,
    week_start: chrono::Weekday,
    min_date: Option<NaiveDate>,
    max_date: Option<NaiveDate>,
    help_message: Option<&'a str>,
    vim_mode: bool,
    formatter: DateFormatter<'a>,
    validators: Vec<Box<dyn DateValidator>>,
    error: Option<ErrorMessage>,
}

impl<'a> DateSelectPrompt<'a> {
    fn new(so: DateSelect<'a>) -> InquireResult<Self> {
        if let Some(min_date) = so.min_date {
            if min_date > so.starting_date {
                return Err(InquireError::InvalidConfiguration(
                    "Min date can not be greater than starting date".into(),
                ));
            }
        }
        if let Some(max_date) = so.max_date {
            if max_date < so.starting_date {
                return Err(InquireError::InvalidConfiguration(
                    "Max date can not be smaller than starting date".into(),
                ));
            }
        }

        Ok(Self {
            message: so.message,
            current_date: so.starting_date,
            min_date: so.min_date,
            max_date: so.max_date,
            week_start: so.week_start,
            help_message: so.help_message,
            vim_mode: so.vim_mode,
            formatter: so.formatter,
            validators: so.validators,
            error: None,
        })
    }

    fn shift_date(&mut self, duration: chrono::Duration) {
        self.update_date(self.current_date.add(duration));
    }

    fn shift_months(&mut self, qty: i32) {
        let date = self.current_date;

        let years = qty / 12;
        let months = qty % 12;

        let new_year = date.year() + years;
        let cur_month = date.month0() as i32;
        let mut new_month = (cur_month + months) % 12;
        if new_month < 0 {
            new_month += 12;
        }

        let new_date = date
            .with_month0(new_month as u32)
            .and_then(|d| d.with_year(new_year));

        if let Some(new_date) = new_date {
            self.update_date(new_date);
        }
    }

    fn update_date(&mut self, new_date: NaiveDate) {
        self.current_date = new_date;
        if let Some(min_date) = self.min_date {
            self.current_date = max(self.current_date, min_date);
        }
        if let Some(max_date) = self.max_date {
            self.current_date = min(self.current_date, max_date);
        }
    }

    fn on_change(&mut self, key: Key) {
        match key {
            Key::Up(KeyModifiers::NONE) => self.shift_date(Duration::weeks(-1)),
            Key::Char('k', KeyModifiers::NONE) if self.vim_mode => {
                self.shift_date(Duration::weeks(-1))
            }

            Key::Down(KeyModifiers::NONE) | Key::Tab => self.shift_date(Duration::weeks(1)),
            Key::Char('j', KeyModifiers::NONE) if self.vim_mode => {
                self.shift_date(Duration::weeks(1))
            }

            Key::Left(KeyModifiers::NONE) => self.shift_date(Duration::days(-1)),
            Key::Char('h', KeyModifiers::NONE) if self.vim_mode => {
                self.shift_date(Duration::days(-1))
            }

            Key::Right(KeyModifiers::NONE) => self.shift_date(Duration::days(1)),
            Key::Char('l', KeyModifiers::NONE) if self.vim_mode => {
                self.shift_date(Duration::days(1))
            }

            Key::Up(KeyModifiers::CONTROL) => self.shift_months(-12),
            Key::Down(KeyModifiers::CONTROL) => self.shift_months(12),
            Key::Left(KeyModifiers::CONTROL) => self.shift_months(-1),
            Key::Right(KeyModifiers::CONTROL) => self.shift_months(1),
            _ => {}
        }
    }

    fn validate_current_answer(&self) -> InquireResult<Validation> {
        for validator in &self.validators {
            match validator.validate(self.current_date) {
                Ok(Validation::Valid) => {}
                Ok(Validation::Invalid(msg)) => return Ok(Validation::Invalid(msg)),
                Err(err) => return Err(InquireError::Custom(err)),
            }
        }

        Ok(Validation::Valid)
    }

    fn cur_answer(&self) -> NaiveDate {
        self.current_date
    }

    fn render<B: DateSelectBackend>(&mut self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        backend.frame_setup()?;

        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        backend.render_calendar_prompt(prompt)?;

        backend.render_calendar(
            get_month(self.current_date.month()),
            self.current_date.year(),
            self.week_start,
            get_current_date(),
            self.current_date,
            self.min_date,
            self.max_date,
        )?;

        if let Some(help_message) = self.help_message {
            backend.render_help_message(help_message)?;
        }

        backend.frame_finish()?;

        Ok(())
    }

    fn prompt<B: DateSelectBackend>(mut self, backend: &mut B) -> InquireResult<NaiveDate> {
        let final_answer: NaiveDate;

        loop {
            self.render(backend)?;

            let key = backend.read_key()?;

            match key {
                Key::Interrupt => interrupt_prompt!(),
                Key::Cancel => cancel_prompt!(backend, self.message),
                Key::Submit | Key::Char(' ', _) => match self.validate_current_answer()? {
                    Validation::Valid => {
                        final_answer = self.cur_answer();
                        break;
                    }
                    Validation::Invalid(msg) => {
                        self.error = Some(msg);
                    }
                },
                key => self.on_change(key),
            }
        }

        let formatted = (self.formatter)(final_answer);

        finish_prompt_with_answer!(backend, self.message, &formatted, final_answer);
    }
}

#[cfg(test)]
#[cfg(feature = "crossterm")]
mod test {
    use crate::{
        date_utils::get_current_date,
        terminal::crossterm::CrosstermTerminal,
        ui::{Backend, RenderConfig},
        validator::Validation,
        DateSelect,
    };
    use chrono::NaiveDate;
    use crossterm::event::{KeyCode, KeyEvent};

    fn default<'a>() -> DateSelect<'a> {
        DateSelect::new("Question?")
    }

    macro_rules! date_test {
        ($name:ident,$input:expr,$output:expr) => {
            date_test! {$name, $input, $output, default()}
        };

        ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
            #[test]
            fn $name() {
                let read: Vec<KeyEvent> = $input.into_iter().map(KeyEvent::from).collect();
                let mut read = read.iter();

                let mut write: Vec<u8> = Vec::new();
                let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
                let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

                let ans = $prompt.prompt_with_backend(&mut backend).unwrap();

                assert_eq!($output, ans);
            }
        };
    }

    date_test!(today_date, vec![KeyCode::Enter], get_current_date());

    date_test!(
        custom_default_date,
        vec![KeyCode::Enter],
        NaiveDate::from_ymd(2021, 1, 9),
        DateSelect::new("Date").with_default(NaiveDate::from_ymd(2021, 1, 9))
    );

    #[test]
    /// Tests that a closure that actually closes on a variable can be used
    /// as a DateSelect validator.
    fn closure_validator() {
        let read: Vec<KeyEvent> = vec![KeyCode::Enter, KeyCode::Left, KeyCode::Enter]
            .into_iter()
            .map(KeyEvent::from)
            .collect();
        let mut read = read.iter();

        let today_date = get_current_date();

        let validator = move |d| {
            if today_date > d {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Date must be in the past".into()))
            }
        };

        let mut write: Vec<u8> = Vec::new();
        let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
        let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

        let ans = DateSelect::new("Question")
            .with_validator(validator)
            .prompt_with_backend(&mut backend)
            .unwrap();

        assert_eq!(today_date.pred(), ans);
    }
}
