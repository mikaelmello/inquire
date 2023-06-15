//! Help message type.

use std::borrow::Cow;

/// Help message type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelpMessage {
    /// No help message displayed.
    None,

    /// Default help message displayed.
    ///
    /// The actual default help message varies depending on the prompt, possibly
    /// including no default message at all.
    Default,

    /// Custom help message displayed. The content is the given string.
    Custom(String),
}

impl HelpMessage {
    /// Returns the help message as a string, or `None` if the help message is `None` or the default
    /// provided is `None`.
    pub(crate) fn into_or_default(self, default: Option<Cow<'_, str>>) -> Option<String> {
        match self {
            Self::None => None,
            Self::Default => default.map(|s| s.into_owned()),
            Self::Custom(s) => Some(s),
        }
    }
}

impl Default for HelpMessage {
    fn default() -> Self {
        Self::Default
    }
}

impl From<Option<&str>> for HelpMessage {
    fn from(val: Option<&str>) -> Self {
        match val {
            Some(val) => Self::Custom(val.to_string()),
            None => Self::None,
        }
    }
}

impl From<&str> for HelpMessage {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_owned())
    }
}

impl From<&mut str> for HelpMessage {
    fn from(s: &mut str) -> Self {
        Self::Custom(s.to_owned())
    }
}

impl From<String> for HelpMessage {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl From<&String> for HelpMessage {
    fn from(s: &String) -> Self {
        Self::Custom(s.clone())
    }
}

impl<'a> From<Cow<'a, str>> for HelpMessage {
    fn from(s: Cow<'a, str>) -> Self {
        Self::Custom(s.into_owned())
    }
}
