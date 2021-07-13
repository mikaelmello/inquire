use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Key {
    Cancel,
    Submit,
    Backspace,
    Tab,
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
        match event {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
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
            } => Self::Tab,
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: _,
            } => Self::Backspace,
            KeyEvent {
                code: KeyCode::Up,
                modifiers: m,
            } => Self::Up(m),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: m,
            } => Self::Down(m),
            KeyEvent {
                code: KeyCode::Left,
                modifiers: m,
            } => Self::Left(m),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: m,
            } => Self::Right(m),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: m,
            } => Self::Char(c, m),
            #[allow(deprecated)]
            KeyEvent {
                code: c,
                modifiers: m,
            } => Self::Any(c, m),
        }
    }
}
