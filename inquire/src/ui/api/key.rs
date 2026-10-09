use bitflags::bitflags;

bitflags! {
    /// Modifier flags for a key press.
    ///
    /// Using the same struct, but without importing, to cut prompts' direct dependencies to crossterm
    /// https://github.com/crossterm-rs/crossterm/blob/e1260446e94e9a8f7809fef61dc1369b6f8d6e12/src/event.rs#L376-L385
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct KeyModifiers: u8 {
        /// Shift key.
        const SHIFT = 0b0000_0001;
        /// Control key.
        const CONTROL = 0b0000_0010;
        /// Alt key.
        const ALT = 0b0000_0100;
        /// Super/Windows key.
        const SUPER = 0b0000_1000;
        /// Hyper key.
        const HYPER = 0b0001_0000;
        /// Meta key.
        const META = 0b0010_0000;
        /// No modifiers.
        const NONE = 0b0000_0000;
    }
}

/// Using the same struct, but without importing, to cut prompts' direct dependencies to crossterm
/// https://github.com/crossterm-rs/crossterm/blob/e1260446e94e9a8f7809fef61dc1369b6f8d6e12/src/event.rs#L376-L385
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Key {
    /// Escape key.
    Escape,
    /// Enter/Return key.
    Enter,
    /// Backspace key.
    Backspace,
    /// Tab key.
    Tab,
    /// Delete key.
    Delete(KeyModifiers),
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp(KeyModifiers),
    /// Page Down key.
    PageDown(KeyModifiers),
    /// Up arrow.
    Up(KeyModifiers),
    /// Down arrow.
    Down(KeyModifiers),
    /// Left arrow.
    Left(KeyModifiers),
    /// Right arrow.
    Right(KeyModifiers),
    /// A character key.
    Char(char, KeyModifiers),
    #[deprecated(note = "If the key you want isn't mapped, please open a PR.")]
    /// Any other key (deprecated).
    Any,
}

#[cfg(test)]
pub(crate) mod key_test {
    use super::{Key, KeyModifiers};

    impl Key {
        /// Helper for tests: converts a string into `Key::Char` events with `KeyModifiers::NONE`.
        pub fn char_keys_from_str(s: &str) -> Vec<Self> {
            s.chars()
                .map(|c| Key::Char(c, KeyModifiers::NONE))
                .collect()
        }
    }
}
