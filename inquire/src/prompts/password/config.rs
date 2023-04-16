use crate::{Password, PasswordDisplayMode};

#[derive(Copy, Clone, Debug)]
pub struct PasswordConfig {
    pub enable_display_toggle: bool,
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
