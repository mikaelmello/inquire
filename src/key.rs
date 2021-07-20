use bitflags::bitflags;
use crossterm::event::{KeyCode, KeyEvent};

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

impl From<crossterm::event::KeyModifiers> for KeyModifiers {
    fn from(m: crossterm::event::KeyModifiers) -> Self {
        Self { bits: m.bits() }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Key {
    Cancel,
    Submit,
    Backspace,
    Tab,
    Delete(KeyModifiers),
    Home,
    End,
    Up(KeyModifiers),
    Down(KeyModifiers),
    Left(KeyModifiers),
    Right(KeyModifiers),
    Char(char, KeyModifiers),
    #[deprecated(note = "Please implement the proper matcher for your key on key.rs")]
    Any(KeyCode, KeyModifiers),
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        for _ in 0..100 {
            println!("{:?}", event);
        }
        match event {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: crossterm::event::KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Esc,
                modifiers: _,
            } => Self::Cancel,
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: _,
            }
            | KeyEvent {
                code: KeyCode::Char('\n'),
                modifiers: _,
            }
            | KeyEvent {
                code: KeyCode::Char('\r'),
                modifiers: _,
            } => Self::Submit,
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: _,
            }
            | KeyEvent {
                code: KeyCode::Char('\t'),
                modifiers: _,
            } => Self::Tab,
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: _,
            } => Self::Backspace,
            KeyEvent {
                code: KeyCode::Delete,
                modifiers: m,
            } => Self::Delete(m.into()),
            KeyEvent {
                code: KeyCode::Home,
                modifiers: _,
            } => Self::Home,
            KeyEvent {
                code: KeyCode::End,
                modifiers: _,
            } => Self::End,
            KeyEvent {
                code: KeyCode::Up,
                modifiers: m,
            } => Self::Up(m.into()),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: m,
            } => Self::Down(m.into()),
            KeyEvent {
                code: KeyCode::Left,
                modifiers: m,
            } => Self::Left(m.into()),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: m,
            } => Self::Right(m.into()),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: m,
            } => Self::Char(c, m.into()),
            #[allow(deprecated)]
            KeyEvent {
                code: c,
                modifiers: m,
            } => Self::Any(c, m.into()),
        }
    }
}
