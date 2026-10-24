use std::{collections::VecDeque, fmt::Display};

use unicode_width::UnicodeWidthChar;

use crate::ansi::{AnsiAware, AnsiAwareChar};
use crate::ui::Styled;

use super::{Terminal, TerminalSize};

pub struct MockTerminal<'a> {
    pub size: TerminalSize,
    pub output: &'a mut VecDeque<MockTerminalToken>,
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
impl<T> From<Styled<T>> for MockTerminalToken
where
    T: Display,
{
    fn from(val: Styled<T>) -> Self {
        MockTerminalToken::Text(Styled::new(val.content.to_string()).with_style_sheet(val.style))
    }
}

pub fn match_text(output: &mut VecDeque<MockTerminalToken>, text: &str) {
    while let Some(actual) = output.pop_front() {
        if let MockTerminalToken::Text(actual) = actual {
            if actual.content == text {
                return;
            } else {
                panic!("Expected text {:?} but found {:?}", text, actual.content);
            }
        }
    }
    panic!("Expected text not found: {:?}", text);
}

pub fn match_token(output: &mut VecDeque<MockTerminalToken>, token: MockTerminalToken) {
    match output.pop_front() {
        Some(actual) => {
            if actual == token {
                return;
            }
            panic!("Expected token {:?} but found {:?}", token, actual);
        }
        None => panic!("Expected token not found: {:?}", token),
    }
}

impl<'a> MockTerminal<'a> {
    pub fn new(output: &'a mut VecDeque<MockTerminalToken>) -> Self {
        Self {
            size: TerminalSize::new(80, 40).unwrap(),
            output,
        }
    }

    pub fn with_size(mut self, size: TerminalSize) -> Self {
        self.size = size;
        self
    }

    pub fn match_text(&mut self, text: &str) {
        match_text(self.output, text);
    }
}

impl<'a> Terminal for MockTerminal<'a> {
    fn get_size(&self) -> std::io::Result<Option<TerminalSize>> {
        Ok(Some(self.size))
    }

    fn write<T: Display>(&mut self, val: T) -> std::io::Result<()> {
        let styled = Styled::new(format!("{val}"));
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

/// What a terminal would show at the end of a rendered frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenSnapshot {
    /// Visible rows, with trailing blanks trimmed and trailing empty rows removed.
    pub rows: Vec<String>,
    /// Cursor position as (row, col).
    pub cursor: (usize, usize),
}

/// Replays [`MockTerminalToken`]s on a minimal emulated screen, following the
/// conventions of common VT terminals (auto-wrap with a pending wrap at the
/// right margin, tabs moving to the next 8-column tab stop, wide chars taking
/// two cells). Styles are ignored and the screen grows downwards instead of
/// scrolling.
///
/// Returns one snapshot for each [`MockTerminalToken::CursorShow`], i.e. one
/// per finished frame.
pub fn emulate_screen<'a>(
    tokens: impl IntoIterator<Item = &'a MockTerminalToken>,
    size: TerminalSize,
) -> Vec<ScreenSnapshot> {
    let width = usize::from(size.width());
    let mut grid: Vec<Vec<char>> = vec![];
    let (mut row, mut col) = (0usize, 0usize);
    let mut pending_wrap = false;
    let mut snapshots = vec![];

    fn ensure_row(grid: &mut Vec<Vec<char>>, row: usize, width: usize) {
        while grid.len() <= row {
            grid.push(vec![' '; width]);
        }
    }

    for token in tokens {
        match token {
            MockTerminalToken::Text(styled) => {
                for piece in styled.content.ansi_aware_chars() {
                    let c = match piece {
                        AnsiAwareChar::Char(c) => c,
                        AnsiAwareChar::AnsiEscapeSequence(_) => continue,
                    };
                    match c {
                        '\r' => {
                            col = 0;
                            pending_wrap = false;
                        }
                        '\n' => {
                            row += 1;
                            pending_wrap = false;
                        }
                        '\t' => {
                            col = ((col / 8 + 1) * 8).min(width - 1);
                        }
                        c => {
                            let w = UnicodeWidthChar::width(c).unwrap_or(0);
                            if w == 0 {
                                continue;
                            }
                            if pending_wrap || col + w > width {
                                row += 1;
                                col = 0;
                                pending_wrap = false;
                            }
                            ensure_row(&mut grid, row, width);
                            grid[row][col] = c;
                            for cell in grid[row].iter_mut().skip(col + 1).take(w - 1) {
                                *cell = '\0';
                            }
                            col += w;
                            if col >= width {
                                col = width - 1;
                                pending_wrap = true;
                            }
                        }
                    }
                }
            }
            MockTerminalToken::ClearLine => {
                ensure_row(&mut grid, row, width);
                grid[row] = vec![' '; width];
            }
            MockTerminalToken::ClearUntilNewLine => {
                ensure_row(&mut grid, row, width);
                for cell in grid[row].iter_mut().skip(col) {
                    *cell = ' ';
                }
            }
            MockTerminalToken::CursorUp(n) => {
                row = row.saturating_sub(usize::from(*n));
                pending_wrap = false;
            }
            MockTerminalToken::CursorDown(n) => {
                row += usize::from(*n);
                pending_wrap = false;
            }
            MockTerminalToken::CursorLeft(n) => {
                col = col.saturating_sub(usize::from(*n));
                pending_wrap = false;
            }
            MockTerminalToken::CursorRight(n) => {
                col = (col + usize::from(*n)).min(width - 1);
                pending_wrap = false;
            }
            MockTerminalToken::CursorMoveToColumn(n) => {
                col = usize::from(*n).min(width - 1);
                pending_wrap = false;
            }
            MockTerminalToken::CursorShow => {
                let mut rows: Vec<String> = grid
                    .iter()
                    .map(|r| {
                        let line: String = r.iter().filter(|c| **c != '\0').collect();
                        line.trim_end().to_string()
                    })
                    .collect();
                while rows.last().is_some_and(String::is_empty) {
                    rows.pop();
                }
                snapshots.push(ScreenSnapshot {
                    rows,
                    cursor: (row, col),
                });
            }
            MockTerminalToken::CursorHide => {}
        }
    }

    snapshots
}
