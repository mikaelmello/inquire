use chrono::{Datelike, Duration, NaiveDate};
use std::{
    cmp::{max, min},
    ops::Add,
};

use crate::{
    date_utils::get_month,
    error::{InquireError, InquireResult},
    formatter::{self, DateFormatter},
    key::{Key, KeyModifiers},
    renderer::Renderer,
    terminal::Terminal,
    validator::DateValidator,
};

/// Presents a message to the user and a date picker for the user to choose from.
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
    pub validators: Vec<DateValidator<'a>>,
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
    pub const DEFAULT_VALIDATORS: Vec<DateValidator<'a>> = vec![];

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
            starting_date: chrono::Local::now().date().naive_local(),
            min_date: Self::DEFAULT_MIN_DATE,
            max_date: Self::DEFAULT_MAX_DATE,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            formatter: Self::DEFAULT_FORMATTER,
            validators: Self::DEFAULT_VALIDATORS,
            week_start: Self::DEFAULT_WEEK_START,
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

    /// Sets the default date.
    pub fn with_default(mut self, default: NaiveDate) -> Self {
        self.starting_date = default;
        self
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

    /// Adds a validator to the collection of validators. You might want to use this feature
    /// in case you need to limit the user to specific choices, such as not allowing weekends.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validator(mut self, validator: DateValidator<'a>) -> Self {
        self.validators.push(validator);
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
    pub fn with_validators(mut self, validators: &[DateValidator<'a>]) -> Self {
        for validator in validators {
            self.validators.push(validator.clone());
        }
        self
    }

    /// Enables or disabled vim_mode.
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: DateFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<NaiveDate> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(
        self,
        renderer: &mut Renderer,
    ) -> InquireResult<NaiveDate> {
        DateSelectPrompt::new(self)?.prompt(renderer)
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
    validators: Vec<DateValidator<'a>>,
    error: Option<String>,
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

    fn get_final_answer(&self) -> Result<NaiveDate, String> {
        for validator in &self.validators {
            match validator(self.current_date) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(self.current_date)
    }

    fn render(&mut self, renderer: &mut Renderer) -> InquireResult<()> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(err) = &self.error {
            renderer.print_error_message(err)?;
        }

        renderer.print_prompt(&prompt, None, None)?;

        renderer.print_calendar_month(
            get_month(self.current_date.month()),
            self.current_date.year(),
            self.week_start,
            chrono::Local::now().date().naive_local(),
            self.current_date,
            self.min_date,
            self.max_date,
        )?;

        if let Some(help_message) = self.help_message {
            renderer.print_help(help_message)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> InquireResult<NaiveDate> {
        let final_answer: NaiveDate;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Cancel => return Err(InquireError::OperationCanceled),
                Key::Submit | Key::Char(' ', _) => match self.get_final_answer() {
                    Ok(answer) => {
                        final_answer = answer;
                        break;
                    }
                    Err(err) => self.error = Some(err),
                },
                key => self.on_change(key),
            }
        }

        let formatted = (self.formatter)(final_answer);

        renderer.cleanup(&self.message, &formatted)?;

        Ok(final_answer)
    }
}

#[cfg(test)]
mod test {
    use crate::{renderer::Renderer, terminal::Terminal, DateSelect};
    use chrono::NaiveDate;
    use crossterm::event::{KeyCode, KeyEvent};
    use ntest::timeout;

    fn default<'a>() -> DateSelect<'a> {
        DateSelect::new("Question?")
    }

    macro_rules! date_test {
        ($name:ident,$input:expr,$output:expr) => {
            date_test! {$name, $input, $output, default()}
        };

        ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
            #[test]
            #[timeout(100)]
            fn $name() {
                let read: Vec<KeyEvent> = $input.into_iter().map(KeyEvent::from).collect();
                let mut read = read.iter();

                let mut write: Vec<u8> = Vec::new();
                let terminal = Terminal::new_with_io(&mut write, &mut read);
                let mut renderer = Renderer::new(terminal).unwrap();

                let ans = $prompt.prompt_with_renderer(&mut renderer).unwrap();

                assert_eq!($output, ans);
            }
        };
    }

    date_test!(
        today_date,
        vec![KeyCode::Enter],
        chrono::Local::now().date().naive_local()
    );

    date_test!(
        custom_default_date,
        vec![KeyCode::Enter],
        NaiveDate::from_ymd(2021, 1, 9),
        DateSelect::new("Date").with_default(NaiveDate::from_ymd(2021, 1, 9))
    );

    #[test]
    #[timeout(100)]
    /// Tests that a closure that actually closes on a variable can be used
    /// as a DateSelect validator.
    fn closure_validator() {
        let read: Vec<KeyEvent> = vec![KeyCode::Enter, KeyCode::Left, KeyCode::Enter]
            .into_iter()
            .map(KeyEvent::from)
            .collect();
        let mut read = read.iter();

        let today_date = chrono::Local::now().date().naive_local();

        let validator = |d| {
            if today_date > d {
                Ok(())
            } else {
                Err(String::from("Date must be in the past"))
            }
        };

        let mut write: Vec<u8> = Vec::new();
        let terminal = Terminal::new_with_io(&mut write, &mut read);
        let mut renderer = Renderer::new(terminal).unwrap();

        let ans = DateSelect::new("Question")
            .with_validator(&validator)
            .prompt_with_renderer(&mut renderer)
            .unwrap();

        assert_eq!(today_date.pred(), ans);
    }
}
