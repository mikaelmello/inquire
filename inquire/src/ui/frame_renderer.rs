use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::io;

use fxhash::FxHasher;
use unicode_width::UnicodeWidthChar;

use super::dimension::Dimension;
use super::{Position, Styled};
use crate::ansi::{AnsiAware, AnsiAwareChar};
use crate::terminal::{Terminal, TerminalSize};

#[derive(Debug, Default)]
struct FrameRow {
    content: Vec<Styled<String>>,
    hash: u64,
}

impl FrameRow {
    pub fn new(content: Vec<Styled<String>>, hash: u64) -> Self {
        Self { content, hash }
    }

    pub fn get_content(&self) -> &[Styled<String>] {
        &self.content
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}

#[derive(Debug)]
struct FrameState {
    /// terminal size when the frame was rendered
    pub terminal_size: TerminalSize,
    /// resulting frame size
    pub frame_size: Dimension,
    /// position to put cursor after writing all present content
    pub expected_cursor_position: Option<Position>,
    /// content and pre-calculated hashes for each rendered line
    /// the length of this vector should be equal to frame_size.height
    pub finished_rows: Vec<FrameRow>,
    pub current_styled: Styled<String>,
    pub current_line: Vec<Styled<String>>,
    pub current_line_width: u16,
    pub current_line_hasher: FxHasher,
}

impl FrameState {
    pub fn new(terminal_size: TerminalSize) -> Self {
        Self {
            terminal_size,
            frame_size: Dimension::new(0, 0),
            finished_rows: Vec::new(),
            current_styled: Styled::default(),
            current_line: Vec::new(),
            current_line_hasher: FxHasher::default(),
            current_line_width: 0,
            expected_cursor_position: None,
        }
    }

    pub fn write(&mut self, value: &Styled<impl AsRef<str> + Display>) {
        self.current_styled.style = value.style;

        for piece in value.content.ansi_aware_chars() {
            piece.hash(&mut self.current_line_hasher);
            value.style.hash(&mut self.current_line_hasher);

            let current_char = match piece {
                AnsiAwareChar::Char(c) => c,
                AnsiAwareChar::AnsiEscapeSequence(_) => {
                    // we don't care for escape sequences when calculating cursor position
                    // and box size
                    continue;
                }
            };

            if current_char == '\n' {
                self.finish_line();
                continue;
            }

            let remaining_width_space = self.terminal_size.width() - self.current_line_width;
            let character_length = UnicodeWidthChar::width(current_char).unwrap_or(0) as u16;

            if character_length > remaining_width_space {
                // the character will (probably) not fit into the current line
                self.finish_line();
            }

            self.current_line_width = self.current_line_width.saturating_add(character_length);
            self.current_styled.content.push(current_char);
        }

        if !self.current_styled.content.is_empty() {
            self.current_line
                .push(std::mem::take(&mut self.current_styled));
        }
    }

    pub fn mark_cursor_position(&mut self, offset: isize) {
        let mut row = self.finished_rows.len() as u16;
        let mut col = self.current_line_width;

        col = col.saturating_add(offset as u16);

        if col >= self.terminal_size.width() {
            col -= self.terminal_size.width();
            row += 1;
        }

        self.expected_cursor_position = Some(Position { row, col });
    }

    pub fn finish(&mut self) {
        self.finish_line();
    }

    pub fn resize_if_needed(&mut self, new_size: TerminalSize) {
        if new_size == self.terminal_size {
            return;
        }

        let mut new_state = Self::new(new_size);
        for row in &self.finished_rows {
            for styled in row.get_content() {
                new_state.write(styled);
            }
            new_state.finish_line();
        }
        for styled in &self.current_line {
            new_state.write(styled);
        }
        new_state.finish_line();

        *self = new_state;
    }

    fn finish_line(&mut self) {
        let current_styled = std::mem::take(&mut self.current_styled);
        self.current_styled.style = current_styled.style;

        if !current_styled.content.is_empty() || !current_styled.style.is_empty() {
            self.current_line.push(current_styled);
        }

        let hasher = std::mem::take(&mut self.current_line_hasher);
        let content = std::mem::take(&mut self.current_line);

        if content.is_empty() {
            return;
        }

        self.finished_rows
            .push(FrameRow::new(content, hasher.finish()));

        self.frame_size = Dimension::new(
            self.frame_size.width().max(self.current_line_width),
            self.finished_rows.len() as u16,
        );

        if !self.current_styled.style.is_empty() {
            self.current_styled
                .style
                .hash(&mut self.current_line_hasher);
        }

        self.current_line_width = 0;
    }
}

#[derive(Debug, Default)]
enum RenderState {
    #[default]
    Initial,
    ActiveRender {
        last_rendered_frame: FrameState,
        current_frame: FrameState,
    },
    Rendered(FrameState),
}

pub struct FrameRenderer<T>
where
    T: Terminal,
{
    terminal: T,
    cursor_position: Position,
    state: RenderState,
}

impl<T> FrameRenderer<T>
where
    T: Terminal,
{
    pub fn new(terminal: T) -> io::Result<Self> {
        Ok(Self {
            terminal,
            cursor_position: Position::default(),
            state: RenderState::Initial,
        })
    }

    pub fn write(&mut self, value: impl Display) -> io::Result<()> {
        self.write_styled(Styled::new(value))
    }

    pub fn write_styled(&mut self, value: Styled<impl Display>) -> io::Result<()> {
        match &mut self.state {
            RenderState::Rendered(_) | RenderState::Initial => {}
            RenderState::ActiveRender { current_frame, .. } => {
                // here we are converting from a generic impl Display to String
                // because we are storing the string content in the frame (we can't store a ref to an object, for example).
                //
                // we pay a little bit in memory/cpu usage for this so we can
                // calculate incremental rendering and cursor position on-the-fly.
                let formatted = format!("{}", value.content);
                let value = value.with_content(formatted);

                current_frame.write(&value);
            }
        }

        Ok(())
    }

    pub fn mark_cursor_position(&mut self, offset: isize) {
        match &mut self.state {
            RenderState::Rendered(_) | RenderState::Initial => {}
            RenderState::ActiveRender { current_frame, .. } => {
                current_frame.mark_cursor_position(offset);
            }
        }
    }

    pub fn start_frame(&mut self) -> io::Result<()> {
        let terminal_size = self.refresh_terminal_size();

        self.state = match std::mem::replace(&mut self.state, RenderState::Initial) {
            RenderState::Initial => RenderState::ActiveRender {
                last_rendered_frame: FrameState::new(terminal_size),
                current_frame: FrameState::new(terminal_size),
            },

            RenderState::Rendered(last_rendered_frame) => RenderState::ActiveRender {
                last_rendered_frame,
                current_frame: FrameState::new(terminal_size),
            },

            RenderState::ActiveRender {
                last_rendered_frame,
                current_frame,
            } => RenderState::ActiveRender {
                last_rendered_frame,
                current_frame,
            },
        };

        Ok(())
    }

    pub fn finish_current_frame(&mut self) -> io::Result<()> {
        let (last_rendered_frame, mut current_frame) = match std::mem::take(&mut self.state) {
            RenderState::Rendered(_) | RenderState::Initial => {
                return Ok(());
            }
            RenderState::ActiveRender {
                last_rendered_frame,
                current_frame,
            } => (last_rendered_frame, current_frame),
        };

        current_frame.finish();

        let rows_to_iterate = std::cmp::max(
            last_rendered_frame.frame_size.height(),
            current_frame.frame_size.height(),
        );

        self.terminal.cursor_hide()?;
        self.move_cursor_to(Position { row: 0, col: 0 })?;

        for i in 0..rows_to_iterate {
            let last_row = last_rendered_frame.finished_rows.get(i as usize);
            let current_row = current_frame.finished_rows.get(i as usize);

            match (last_row, current_row) {
                (Some(last_row), Some(current_row)) => {
                    if last_row.hash() != current_row.hash() {
                        for styled in current_row.get_content() {
                            self.terminal.write_styled(styled)?;
                        }
                        self.terminal.clear_until_new_line()?;
                    }
                }
                (Some(_), None) => {
                    self.terminal.clear_line()?;
                }
                (None, Some(current_row)) => {
                    for styled in current_row.get_content() {
                        self.terminal.write_styled(styled)?;
                    }
                }
                (None, None) => {
                    // unreachable, but we don't want to panic live :)
                    #[cfg(test)]
                    unreachable!(
                        "frame_size should never be larger then finished_rows for both frames"
                    )
                }
            }

            self.terminal.write("\r")?;
            self.cursor_position.col = 0;
            if i + 1 < rows_to_iterate {
                self.terminal.write("\n")?;
                self.cursor_position.row += 1;
            }
        }

        if let Some(expected_cursor_position) = current_frame.expected_cursor_position {
            self.move_cursor_to(expected_cursor_position)?;
        }

        self.terminal.cursor_show()?;
        self.terminal.flush()?;

        self.state = RenderState::Rendered(current_frame);

        Ok(())
    }

    fn move_cursor_to_end_position(&mut self) -> io::Result<()> {
        self.refresh_terminal_size();

        let last_rendered = match &mut self.state {
            RenderState::Initial => return Ok(()),
            RenderState::ActiveRender {
                last_rendered_frame,
                ..
            }
            | RenderState::Rendered(last_rendered_frame) => last_rendered_frame,
        };

        let end_position = Position {
            col: 0,
            row: last_rendered.frame_size.height(),
        };

        self.move_cursor_to(end_position)?;

        Ok(())
    }

    fn move_cursor_to(&mut self, position: Position) -> io::Result<()> {
        let current_cursor_position = self.cursor_position;

        match current_cursor_position.row.cmp(&position.row) {
            Ordering::Greater => {
                self.terminal
                    .cursor_up(current_cursor_position.row - position.row)?;
            }
            Ordering::Less => {
                self.terminal
                    .cursor_down(position.row - current_cursor_position.row)?;
            }
            Ordering::Equal => {}
        }

        match current_cursor_position.col.cmp(&position.col) {
            Ordering::Greater => {
                self.terminal
                    .cursor_left(current_cursor_position.col - position.col)?;
            }
            Ordering::Less => {
                self.terminal
                    .cursor_right(position.col - current_cursor_position.col)?;
            }
            Ordering::Equal => {}
        }

        self.cursor_position = position;

        Ok(())
    }

    fn refresh_terminal_size(&mut self) -> TerminalSize {
        // not properly handling resizes is better than panicking, so when
        // getting the terminal size fails, we assume we're on a terminal
        // that will always have enough space
        let terminal_size = self
            .terminal
            .get_size()
            .unwrap_or(TerminalSize::new(1000, 1000));

        if terminal_size.width() < self.cursor_position.col {
            let new_line_offset = self.cursor_position.col / terminal_size.width();
            let new_col = self.cursor_position.col % terminal_size.width();
            self.cursor_position = Position {
                row: self.cursor_position.row + new_line_offset,
                col: new_col,
            };
        }

        match &mut self.state {
            RenderState::Initial => {}
            RenderState::ActiveRender {
                current_frame,
                last_rendered_frame,
            } => {
                last_rendered_frame.resize_if_needed(terminal_size);
                current_frame.resize_if_needed(terminal_size);
            }
            RenderState::Rendered(last_rendered_frame) => {
                last_rendered_frame.resize_if_needed(terminal_size);
            }
        };

        terminal_size
    }
}

impl<T> Drop for FrameRenderer<T>
where
    T: Terminal,
{
    fn drop(&mut self) {
        let _unused = self.move_cursor_to_end_position();
        let _unused = self.terminal.cursor_show();
        let _unused = self.terminal.flush();
    }
}
