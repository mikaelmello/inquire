use crate::ui::{InnerPromptAction, Key, KeyModifiers};

use super::config::DateSelectConfig;

pub enum DateSelectAction {
    GoToPrevDay,
    GoToNextDay,
    GoToPrevWeek,
    GoToNextWeek,
    GoToPrevMonth,
    GoToNextMonth,
    GoToPrevYear,
    GoToNextYear,
}

impl InnerPromptAction<DateSelectConfig> for DateSelectAction {
    fn map_key(key: Key, config: DateSelectConfig) -> Option<Self> {
        if config.vim_mode {
            match key {
                Key::Char('k', KeyModifiers::NONE) if config.vim_mode => {
                    return Some(DateSelectAction::GoToPrevWeek)
                }
                Key::Char('j', KeyModifiers::NONE) if config.vim_mode => {
                    return Some(DateSelectAction::GoToNextWeek)
                }
                Key::Char('h', KeyModifiers::NONE) if config.vim_mode => {
                    return Some(DateSelectAction::GoToPrevDay)
                }
                Key::Char('l', KeyModifiers::NONE) if config.vim_mode => {
                    return Some(DateSelectAction::GoToNextDay)
                }
                _ => (),
            }
        }

        match key {
            Key::Left(KeyModifiers::NONE) => return Some(DateSelectAction::GoToPrevDay),
            Key::Right(KeyModifiers::NONE) => return Some(DateSelectAction::GoToNextDay),
            Key::Up(KeyModifiers::NONE) => return Some(DateSelectAction::GoToPrevWeek),
            Key::Down(KeyModifiers::NONE) | Key::Tab => {
                return Some(DateSelectAction::GoToNextWeek)
            }
            Key::Left(KeyModifiers::CONTROL) => return Some(DateSelectAction::GoToPrevMonth),
            Key::Right(KeyModifiers::CONTROL) => return Some(DateSelectAction::GoToNextMonth),
            Key::Up(KeyModifiers::CONTROL) => return Some(DateSelectAction::GoToPrevYear),
            Key::Down(KeyModifiers::CONTROL) => return Some(DateSelectAction::GoToNextYear),
            _ => return None,
        };
    }
}
