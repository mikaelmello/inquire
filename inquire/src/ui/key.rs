use bitflags::bitflags;

// Using the same struct, but without importing, to cut prompts' direct dependencies to crossterm
// https://github.com/crossterm-rs/crossterm/blob/e1260446e94e9a8f7809fef61dc1369b6f8d6e12/src/event.rs#L376-L385
bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
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

pub trait InnerPromptAction<C>
where
    Self: Sized,
{
    fn map_key(key: Key, config: C) -> Option<Self>;
}

pub enum PromptAction<A>
where
    A: InnerPromptAction<C>,
{
    Cancel,
    Interrupt,
    Submit,
    Inner(A),
    None,
}

impl<A, C> PromptAction<A, C>
where
    A: InnerPromptAction<C>,
{
    pub fn from_key(key: Key, config: C) -> Self {
        match key {
            Key::Cancel => PromptAction::Cancel,
            Key::Interrupt => PromptAction::Interrupt,
            Key::Submit => PromptAction::Submit,
            key => match A::map_key(key, config) {
                Some(inner) => PromptAction::Inner(inner),
                None => PromptAction::None,
            },
        }
    }
}
