use crate::{
    ui::{Key, KeyModifiers},
    ActionMapper, BuiltinActionMapper, DateSelect, InnerAction,
};

use super::config::DateSelectConfig;

/// Set of actions for a DateSelectPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum DateSelectAction {
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

impl InnerAction<DateSelectConfig> for DateSelectAction {
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

pub(crate) struct BuiltinDateSelectActionMapper {
    config: DateSelectConfig,
}

impl BuiltinDateSelectActionMapper {
    pub(crate) fn new(
        prompt: &DateSelect<'_>,
    ) -> BuiltinActionMapper<DateSelectAction, BuiltinDateSelectActionMapper> {
        BuiltinActionMapper::new(Self {
            config: prompt.into(),
        })
    }
}

impl ActionMapper<DateSelectAction> for BuiltinDateSelectActionMapper {
    fn get_action(&self, key: Key) -> Option<DateSelectAction> {
        if self.config.vim_mode {
            let action = match key {
                Key::Char('k', KeyModifiers::NONE) => Some(DateSelectAction::GoToPrevWeek),
                Key::Char('j', KeyModifiers::NONE) => Some(DateSelectAction::GoToNextWeek),
                Key::Char('h', KeyModifiers::NONE) => Some(DateSelectAction::GoToPrevDay),
                Key::Char('l', KeyModifiers::NONE) => Some(DateSelectAction::GoToNextDay),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        let action = match key {
            Key::Left(KeyModifiers::NONE) => DateSelectAction::GoToPrevDay,
            Key::Right(KeyModifiers::NONE) => DateSelectAction::GoToNextDay,
            Key::Up(KeyModifiers::NONE) => DateSelectAction::GoToPrevWeek,
            Key::Down(KeyModifiers::NONE) | Key::Tab => DateSelectAction::GoToNextWeek,
            Key::Left(KeyModifiers::CONTROL) => DateSelectAction::GoToPrevMonth,
            Key::Right(KeyModifiers::CONTROL) => DateSelectAction::GoToNextMonth,
            Key::Up(KeyModifiers::CONTROL) => DateSelectAction::GoToPrevYear,
            Key::Down(KeyModifiers::CONTROL) => DateSelectAction::GoToNextYear,
            _ => return None,
        };

        Some(action)
    }
}
