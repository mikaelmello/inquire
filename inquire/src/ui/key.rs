use bitflags::bitflags;

// Using the same struct, but without importing, to cut prompts' direct dependencies to crossterm
// https://github.com/crossterm-rs/crossterm/blob/e1260446e94e9a8f7809fef61dc1369b6f8d6e12/src/event.rs#L376-L385
bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const SUPER = 0b0000_1000;
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
        const NONE = 0b0000_0000;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Key {
    Escape,
    Enter,
    Backspace,
    Tab,
    Delete(KeyModifiers),
    Home,
    End,
    PageUp(KeyModifiers),
    PageDown(KeyModifiers),
    Up(KeyModifiers),
    Down(KeyModifiers),
    Left(KeyModifiers),
    Right(KeyModifiers),
    Char(char, KeyModifiers),
    #[deprecated(note = "If the key you want isn't mapped, please open a PR.")]
    Any,
}

#[cfg(test)]
pub(crate) mod key_test {
    use super::{Key, KeyModifiers};

    impl Key {
        pub fn char_keys_from_str(s: &str) -> Vec<Self> {
            s.chars()
                .map(|c| Key::Char(c, KeyModifiers::NONE))
                .collect()
        }
    }
}
