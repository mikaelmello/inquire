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

    fn from_key(key: Key, _: &DateSelectConfig) -> Option<Self> {
        let action = match key {
            Key::Left(KeyModifiers::NONE) // standard
            | Key::Char('b', KeyModifiers::CONTROL) // emacs
            | Key::Char('h', KeyModifiers::NONE) // vim
            => Self::GoToPrevDay,

            Key::Right(KeyModifiers::NONE) // standard
            | Key::Char('f', KeyModifiers::CONTROL) // emacs
            | Key::Char('l', KeyModifiers::NONE) // vim
            => Self::GoToNextDay,

            Key::Up(KeyModifiers::NONE) // standard
            | Key::Char('p', KeyModifiers::CONTROL) // emacs
            | Key::Char('k', KeyModifiers::NONE) // vim
             => Self::GoToPrevWeek,

            Key::Down(KeyModifiers::NONE) // standard
            | Key::Char('n', KeyModifiers::CONTROL) // emacs
            | Key::Char('j', KeyModifiers::NONE) // vim
            | Key::Tab // not sure? keeping it for compatibility reasons now
            => Self::GoToNextWeek,

            Key::PageUp(KeyModifiers::NONE) // standard
            | Key::Char('[', KeyModifiers::NONE) // alternative when page up is not available
            | Key::Left(_) // alternative 2, when the left above with no modifiers is not matched
            | Key::Char('v' | 'V', KeyModifiers::ALT | KeyModifiers::META) // emacs
            | Key::Char('b' | 'B', _) // vim, ideally ctrl-b should be used, but it's not available due to emacs
             => Self::GoToPrevMonth,

            Key::PageDown(KeyModifiers::NONE) // standard
            | Key::Char(']', KeyModifiers::NONE) // alternative when page down is not available
            | Key::Right(_) // alternative 2, when the right above with no modifiers is not matched
            | Key::Char('v' | 'V', KeyModifiers::CONTROL) // emacs
            | Key::Char('f' | 'F', _) // vim, ideally ctrl-f should be used, but it's not available due to emacs
             => Self::GoToNextMonth,

            Key::PageUp(_) // standard, when the above with no modifiers is not matched
            | Key::Char('{' | '[', _) // alternative when page up is not available
            | Key::Up(_) // alternative 2, when the up above with no modifiers is not matched
            => Self::GoToPrevYear,

            Key::PageDown(_) // standard, when the above with no modifiers is not matched
            | Key::Char('}' | ']', _) // alternative when page down is not available
            | Key::Down(_) // alternative 2, when the down above with no modifiers is not matched
            => Self::GoToNextYear,

            _ => return None,
        };

        Some(action)
    }
}
