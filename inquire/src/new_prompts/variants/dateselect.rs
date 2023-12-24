use std::{
    cmp::{max, min},
    ops::Add,
};

use chrono::{Datelike, Duration, NaiveDate};

use crate::{
    api::DateSelectPromptAction,
    date_utils::{get_current_date, get_month},
    error::InquireResult,
    new_prompts::{action::ParseKey, action_result::ActionResult, base::PromptImpl},
    ui::{date::DateSelectBackend, Key, KeyModifiers},
    DateSelectConfig,
};

impl ParseKey for DateSelectPromptAction {
    fn from_key(key: Key) -> Option<Self> {
        match key {
            Key::Left(KeyModifiers::NONE) | Key::Char('b', KeyModifiers::CONTROL) => {
                Some(Self::GoToPrevDay)
            }
            Key::Right(KeyModifiers::NONE) | Key::Char('f', KeyModifiers::CONTROL) => {
                Some(Self::GoToNextDay)
            }
            Key::Up(KeyModifiers::NONE) | Key::Char('p', KeyModifiers::CONTROL) => {
                Some(Self::GoToPrevWeek)
            }
            Key::Down(KeyModifiers::NONE) | Key::Char('n', KeyModifiers::CONTROL) | Key::Tab => {
                Some(Self::GoToNextWeek)
            }
            Key::Left(KeyModifiers::CONTROL) => Some(Self::GoToPrevMonth),
            Key::Right(KeyModifiers::CONTROL) => Some(Self::GoToNextMonth),
            Key::Up(KeyModifiers::CONTROL) => Some(Self::GoToPrevYear),
            Key::Down(KeyModifiers::CONTROL) => Some(Self::GoToNextYear),
            _ => None,
        }
    }
}

pub struct DateSelectPrompt {
    pub config: DateSelectConfig,
    pub current_date: NaiveDate,
}

impl DateSelectPrompt {
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
}

impl<'a, B> PromptImpl<'a, B> for DateSelectPrompt
where
    B: DateSelectBackend,
{
    type Action = DateSelectPromptAction;
    type Output = NaiveDate;
    type OutputAsArgument = NaiveDate;

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

    fn render(&self, message: &str, backend: &mut B) -> InquireResult<()> {
        backend.render_calendar_prompt(message)?;

        backend.render_calendar(
            get_month(self.current_date.month()),
            self.current_date.year(),
            self.config.week_start,
            get_current_date(),
            self.current_date,
            self.config.min_date,
            self.config.max_date,
        )?;

        Ok(())
    }

    fn current_submission(&self) -> Self::OutputAsArgument {
        self.current_date
    }

    fn into_output(self) -> Self::Output {
        self.current_date
    }
}
