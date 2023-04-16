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
                Key::Char('k', KeyModifiers::NONE) => Some(DateSelectPromptAction::GoToPrevWeek),
                Key::Char('j', KeyModifiers::NONE) => Some(DateSelectPromptAction::GoToNextWeek),
                Key::Char('h', KeyModifiers::NONE) => Some(DateSelectPromptAction::GoToPrevDay),
                Key::Char('l', KeyModifiers::NONE) => Some(DateSelectPromptAction::GoToNextDay),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        match key {
            Key::Left(KeyModifiers::NONE) => Some(DateSelectPromptAction::GoToPrevDay),
            Key::Right(KeyModifiers::NONE) => Some(DateSelectPromptAction::GoToNextDay),
            Key::Up(KeyModifiers::NONE) => Some(DateSelectPromptAction::GoToPrevWeek),
            Key::Down(KeyModifiers::NONE) | Key::Tab => Some(DateSelectPromptAction::GoToNextWeek),
            Key::Left(KeyModifiers::CONTROL) => Some(DateSelectPromptAction::GoToPrevMonth),
            Key::Right(KeyModifiers::CONTROL) => Some(DateSelectPromptAction::GoToNextMonth),
            Key::Up(KeyModifiers::CONTROL) => Some(DateSelectPromptAction::GoToPrevYear),
            Key::Down(KeyModifiers::CONTROL) => Some(DateSelectPromptAction::GoToNextYear),
            _ => None,
        }
    }
}
