use crate::{Password, PasswordDisplayMode};

/// Configuration settings used in the execution of a PasswordPrompt.
#[derive(Copy, Clone, Debug)]
pub struct PasswordConfig {
    /// Whether to allow the user to toggle the display mode of the password.
    pub enable_display_toggle: bool,
    /// The initial display mode of the password.
    pub display_mode: PasswordDisplayMode,
}

impl From<&Password<'_>> for PasswordConfig {
    fn from(value: &Password<'_>) -> Self {
        Self {
            enable_display_toggle: value.enable_display_toggle,
            display_mode: value.display_mode,
        }
    }
}
