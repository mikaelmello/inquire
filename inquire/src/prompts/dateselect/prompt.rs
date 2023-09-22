use std::{
    cmp::{max, min},
    ops::Add,
};

use chrono::{Datelike, Duration, NaiveDate};

use crate::{
    date_utils::{get_current_date, get_month},
    error::InquireResult,
    formatter::DateFormatter,
    prompts::prompt::{ActionResult, Prompt},
    ui::date::DateSelectBackend,
    validator::{DateValidator, ErrorMessage, Validation},
    DateSelect, InquireError,
};

use super::{action::DateSelectPromptAction, config::DateSelectConfig};

pub struct DateSelectPrompt<'a> {
    message: &'a str,
    config: DateSelectConfig,
    current_date: NaiveDate,
    help_message: Option<&'a str>,
    formatter: DateFormatter<'a>,
    validators: Vec<Box<dyn DateValidator>>,
    marked_dates: &'a [NaiveDate],
    error: Option<ErrorMessage>,
}

impl<'a> DateSelectPrompt<'a> {
    pub fn new(so: DateSelect<'a>) -> InquireResult<Self> {
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
            config: (&so).into(),
            help_message: so.help_message,
            formatter: so.formatter,
            validators: so.validators,
            marked_dates: so.marked_dates,
            error: None,
        })
    }

    fn shift_date(&mut self, duration: Duration) -> ActionResult {
        self.update_date(self.current_date.add(duration))
    }

    fn shift_months(&mut self, qty: i32) -> ActionResult {
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
            self.update_date(new_date)
        } else {
            ActionResult::Clean
        }
    }

    fn update_date(&mut self, new_date: NaiveDate) -> ActionResult {
        if self.current_date == new_date {
            return ActionResult::Clean;
        }

        self.current_date = new_date;
        if let Some(min_date) = self.config.min_date {
            self.current_date = max(self.current_date, min_date);
        }
        if let Some(max_date) = self.config.max_date {
            self.current_date = min(self.current_date, max_date);
        }

        ActionResult::NeedsRedraw
    }

    fn validate_current_answer(&self) -> InquireResult<Validation> {
        for validator in &self.validators {
            match validator.validate(self.cur_answer()) {
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
}

impl<'a, B> Prompt<B> for DateSelectPrompt<'a>
where
    B: DateSelectBackend,
{
    type Config = DateSelectConfig;
    type InnerAction = DateSelectPromptAction;
    type Output = NaiveDate;

    fn message(&self) -> &str {
        self.message
    }

    fn format_answer(&self, answer: &NaiveDate) -> String {
        (self.formatter)(*answer)
    }

    fn config(&self) -> &DateSelectConfig {
        &self.config
    }

    fn submit(&mut self) -> InquireResult<Option<NaiveDate>> {
        let answer = match self.validate_current_answer()? {
            Validation::Valid => Some(self.cur_answer()),
            Validation::Invalid(msg) => {
                self.error = Some(msg);
                None
            }
        };

        Ok(answer)
    }

    fn handle(&mut self, action: DateSelectPromptAction) -> InquireResult<ActionResult> {
        let result = match action {
            DateSelectPromptAction::GoToPrevWeek => self.shift_date(Duration::weeks(-1)),
            DateSelectPromptAction::GoToNextWeek => self.shift_date(Duration::weeks(1)),
            DateSelectPromptAction::GoToPrevDay => self.shift_date(Duration::days(-1)),
            DateSelectPromptAction::GoToNextDay => self.shift_date(Duration::days(1)),
            DateSelectPromptAction::GoToPrevYear => self.shift_months(-12),
            DateSelectPromptAction::GoToNextYear => self.shift_months(12),
            DateSelectPromptAction::GoToPrevMonth => self.shift_months(-1),
            DateSelectPromptAction::GoToNextMonth => self.shift_months(1),
        };

        Ok(result)
    }

    fn render(&self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        backend.render_calendar_prompt(prompt)?;

        backend.render_calendar(
            get_month(self.current_date.month()),
            self.current_date.year(),
            self.config.week_start,
            get_current_date(),
            self.current_date,
            self.config.min_date,
            self.config.max_date,
            self.marked_dates,
        )?;

        if let Some(help_message) = self.help_message {
            backend.render_help_message(help_message)?;
        }

        Ok(())
    }
}
