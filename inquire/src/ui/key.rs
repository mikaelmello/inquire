use bitflags::bitflags;

// Using the same struct, but without importing, to cut prompts' direct dependencies to crossterm
// https://github.com/crossterm-rs/crossterm/blob/e1260446e94e9a8f7809fef61dc1369b6f8d6e12/src/event.rs#L376-L385
bitflags! {
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const NONE = 0b0000_0000;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Key {
    Cancel,
    Interrupt,
    Submit,
    Backspace,
    Tab,
    Delete(KeyModifiers),
    Home,
    End,
    PageUp,
    PageDown,
    Up(KeyModifiers),
    Down(KeyModifiers),
    Left(KeyModifiers),
    Right(KeyModifiers),
    Char(char, KeyModifiers),
    #[deprecated(note = "Please implement the proper matcher for your key on key.rs")]
    Any,
}
