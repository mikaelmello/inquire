use chrono::NaiveDate;

use crate::{
    config::get_configuration,
    new_prompts::variants::dateselect::{DateSelectConfig, DateSelectPrompt},
};

use super::common::CommonConfig;

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
    common: CommonConfig<'a, NaiveDate>,
    config: DateSelectConfig,
}

impl<'a> DateSelect<'a> {
    /// Default help message.
    const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("arrows to move, with ctrl to move months and years, enter to select");

    const DEFAULT_DATE_FORMATTER: &dyn Fn(&NaiveDate) -> String =
        &|date: &NaiveDate| date.format("%B %-e, %Y").to_string();

    common_config_builder_methods!(NaiveDate);

    /// Creates a [DateSelect] with the provided message, along with default configuration values.
    pub fn new(message: &'a str) -> Self {
        Self {
            common: CommonConfig {
                message: message.into(),
                help_message: Self::DEFAULT_HELP_MESSAGE.map(String::from),
                formatter: Box::new(Self::DEFAULT_DATE_FORMATTER),
                validators: vec![],
                render_config: get_configuration(),
            },
            config: DateSelectConfig::default(),
        }
    }

    /// Sets the week start.
    pub fn with_week_start(mut self, week_start: chrono::Weekday) -> Self {
        self.config.week_start = week_start;
        self
    }

    /// Sets the min date.
    pub fn with_min_date(mut self, min_date: NaiveDate) -> Self {
        self.config.min_date = Some(min_date);
        self
    }

    fn inner_impl(&self) -> DateSelectPrompt {
        DateSelectPrompt {
            config: self.config,
            current_date: self.config.starting_date,
        }
    }
}

/// Set of actions for a DateSelectPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum DateSelectPromptAction {
    /// Move day cursor to the previous day.
    GoToPrevDay,
    /// Move day cursor to the next day.
    GoToNextDay,
    /// Move day cursor to the previous week.
    GoToPrevWeek,
    /// Move day cursor to the next week.
    GoToNextWeek,
    /// Move day cursor to the previous month.
    GoToPrevMonth,
    /// Move day cursor to the next month.
    GoToNextMonth,
    /// Move day cursor to the previous year.
    GoToPrevYear,
    /// Move day cursor to the next year.
    GoToNextYear,
}
