use crate::ui::{InnerAction, Key, KeyModifiers};

use super::config::DateSelectConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum DateSelectPromptAction {
    GoToPrevDay,
    GoToNextDay,
    GoToPrevWeek,
    GoToNextWeek,
    GoToPrevMonth,
    GoToNextMonth,
    GoToPrevYear,
    GoToNextYear,
}

impl InnerAction<DateSelectConfig> for DateSelectPromptAction {
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
            Key::Left(KeyModifiers::NONE) => Self::GoToPrevDay,
            Key::Right(KeyModifiers::NONE) => Self::GoToNextDay,
            Key::Up(KeyModifiers::NONE) => Self::GoToPrevWeek,
            Key::Down(KeyModifiers::NONE) | Key::Tab => Self::GoToNextWeek,
            Key::Left(KeyModifiers::CONTROL) => Self::GoToPrevMonth,
            Key::Right(KeyModifiers::CONTROL) => Self::GoToNextMonth,
            Key::Up(KeyModifiers::CONTROL) => Self::GoToPrevYear,
            Key::Down(KeyModifiers::CONTROL) => Self::GoToNextYear,
            _ => return None,
        };

        Some(action)
    }
}
