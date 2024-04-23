use std::{collections::VecDeque, fmt::Display};

use crate::ui::{Key, Styled};

use super::{Terminal, TerminalSize};

pub struct MockTerminal {
    pub size: TerminalSize,
    pub input: VecDeque<Key>,
    pub output: VecDeque<MockTerminalToken>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MockTerminalToken {
    Text(Styled<String>),
    ClearLine,
    ClearUntilNewLine,
    CursorHide,
    CursorShow,
    CursorUp(u16),
    CursorDown(u16),
    CursorLeft(u16),
    CursorRight(u16),
    CursorMoveToColumn(u16),
}

impl<T> From<T> for MockTerminalToken
where
    T: Display,
{
    fn from(val: T) -> Self {
        MockTerminalToken::Text(Styled::new(val.to_string()))
    }
}

impl MockTerminal {
    pub fn new() -> Self {
        Self {
            size: TerminalSize::new(80, 40),
            input: VecDeque::new(),
            output: VecDeque::new(),
        }
    }

    pub fn with_size(mut self, size: TerminalSize) -> Self {
        self.size = size;
        self
    }

    pub fn find_and_expect_token(&mut self, token: MockTerminalToken) {
        while let Some(actual) = self.output.pop_front() {
            if actual == token {
                return;
            }
        }

        panic!("Expected token not found: {:?}", token);
    }
}

impl Terminal for MockTerminal {
    fn get_size(&self) -> std::io::Result<TerminalSize> {
        Ok(self.size)
    }

    fn write<T: Display>(&mut self, val: T) -> std::io::Result<()> {
        let styled = Styled::new(format!("{}", val));
        let token = MockTerminalToken::Text(styled);
        self.output.push_back(token);
        Ok(())
    }

    fn write_styled<T: Display>(&mut self, val: &Styled<T>) -> std::io::Result<()> {
        let styled = Styled::new(format!("{}", val.content)).with_style_sheet(val.style);
        let token = MockTerminalToken::Text(styled);
        self.output.push_back(token);
        Ok(())
    }

    fn clear_line(&mut self) -> std::io::Result<()> {
        let token = MockTerminalToken::ClearLine;
        self.output.push_back(token);
        Ok(())
    }

    fn clear_until_new_line(&mut self) -> std::io::Result<()> {
        let token = MockTerminalToken::ClearUntilNewLine;
        self.output.push_back(token);
        Ok(())
    }

    fn cursor_hide(&mut self) -> std::io::Result<()> {
        let token = MockTerminalToken::CursorHide;
        self.output.push_back(token);
        Ok(())
    }

    fn cursor_show(&mut self) -> std::io::Result<()> {
        let token = MockTerminalToken::CursorShow;
        self.output.push_back(token);
        Ok(())
    }

    fn cursor_up(&mut self, cnt: u16) -> std::io::Result<()> {
        let token = MockTerminalToken::CursorUp(cnt);
        self.output.push_back(token);
        Ok(())
    }

    fn cursor_down(&mut self, cnt: u16) -> std::io::Result<()> {
        let token = MockTerminalToken::CursorDown(cnt);
        self.output.push_back(token);
        Ok(())
    }

    fn cursor_left(&mut self, cnt: u16) -> std::io::Result<()> {
        let token = MockTerminalToken::CursorLeft(cnt);
        self.output.push_back(token);
        Ok(())
    }

    fn cursor_right(&mut self, cnt: u16) -> std::io::Result<()> {
        let token = MockTerminalToken::CursorRight(cnt);
        self.output.push_back(token);
        Ok(())
    }

    fn cursor_move_to_column(&mut self, idx: u16) -> std::io::Result<()> {
        let token = MockTerminalToken::CursorMoveToColumn(idx);
        self.output.push_back(token);
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
