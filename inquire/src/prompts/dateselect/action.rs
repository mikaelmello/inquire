use crate::{
    ui::{Key, KeyModifiers},
    InnerAction,
};

use super::config::DateSelectConfig;

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

impl InnerAction for DateSelectPromptAction {
    type Config = DateSelectConfig;

    fn from_key(key: Key, config: &DateSelectConfig) -> Option<Self> {
        if config.vim_mode {
            let action = match key {
                Key::Char('k', KeyModifiers::NONE) => Some(Self::GoToPrevWeek),
                Key::Char('j', KeyModifiers::NONE) => Some(Self::GoToNextWeek),
                Key::Char('h', KeyModifiers::NONE) => Some(Self::GoToPrevDay),
                Key::Char('l', KeyModifiers::NONE) => Some(Self::GoToNextDay),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        let action = match key {
            Key::Left(KeyModifiers::NONE) | Key::Char('b', KeyModifiers::CONTROL) => {
                Self::GoToPrevDay
            }
            Key::Right(KeyModifiers::NONE) | Key::Char('f', KeyModifiers::CONTROL) => {
                Self::GoToNextDay
            }
            Key::Up(KeyModifiers::NONE) | Key::Char('p', KeyModifiers::CONTROL) => {
                Self::GoToPrevWeek
            }
            Key::Down(KeyModifiers::NONE) | Key::Char('n', KeyModifiers::CONTROL) | Key::Tab => {
                Self::GoToNextWeek
            }
            Key::Left(KeyModifiers::CONTROL) => Self::GoToPrevMonth,
            Key::Right(KeyModifiers::CONTROL) => Self::GoToNextMonth,
            Key::Up(KeyModifiers::CONTROL) => Self::GoToPrevYear,
            Key::Down(KeyModifiers::CONTROL) => Self::GoToNextYear,
            _ => return None,
        };

        Some(action)
    }
}
