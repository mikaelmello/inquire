use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::dimension::Dimension;
use super::{Position, Styled};
use crate::ansi::{AnsiAware, AnsiAwareChar};
use crate::terminal::{Terminal, TerminalSize};

/// Distance between tab stops, in columns.
///
/// Tabs are expanded to spaces when rendering so that the renderer knows
/// exactly how many columns they occupy, independently of the terminal's own
/// tab stop configuration.
const TAB_STOP_WIDTH: u16 = 8;

/// Number of spaces a tab expands to when it starts at `col` of a row that is
/// `terminal_width` columns wide.
///
/// Like a terminal, the tab moves to the next tab stop, but never past the
/// right edge of the row.
fn tab_expansion_width(col: u16, terminal_width: u16) -> u16 {
    let next_tab_stop = (col / TAB_STOP_WIDTH)
        .saturating_add(1)
        .saturating_mul(TAB_STOP_WIDTH);

    next_tab_stop.min(terminal_width).saturating_sub(col)
}

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
    pub current_line_hasher: DefaultHasher,
}

impl FrameState {
    pub fn new(terminal_size: TerminalSize) -> Self {
        Self {
            terminal_size,
            frame_size: Dimension::new(0, 0),
            finished_rows: Vec::new(),
            current_styled: Styled::default(),
            current_line: Vec::new(),
            current_line_hasher: DefaultHasher::default(),
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
                AnsiAwareChar::AnsiEscapeSequence(seq) => {
                    // we don't care for escape sequences when calculating cursor position
                    // and box size
                    self.current_styled.content.push_str(seq);
                    continue;
                }
            };

            if current_char == '\n' {
                self.finish_line();
                continue;
            }

            let remaining_width_space = self
                .terminal_size
                .width()
                .saturating_sub(self.current_line_width);

            if current_char == '\t' {
                if remaining_width_space == 0 {
                    // a full row has no room for the tab, it starts the next one
                    self.finish_line();
                }

                let tab_width =
                    tab_expansion_width(self.current_line_width, self.terminal_size.width());
                self.current_line_width = self.current_line_width.saturating_add(tab_width);
                for _ in 0..tab_width {
                    self.current_styled.content.push(' ');
                }
                continue;
            }

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
        let row = self.finished_rows.len() as u16;
        let col = self.current_line_width.saturating_add(offset as u16);

        let row_offset = col / self.terminal_size.width();
        let row = row + row_offset;
        let col = col % self.terminal_size.width();

        self.expected_cursor_position = Some(Position { row, col });
    }

    /// Marks the cursor position as the end of `text`, if `text` were written
    /// starting at the current position. Tabs are measured the same way as
    /// in [`FrameState::write`].
    pub fn mark_cursor_position_after(&mut self, text: &str) {
        let terminal_width = usize::from(self.terminal_size.width());
        let line_start = usize::from(self.current_line_width);
        let mut offset = 0usize;

        for (i, segment) in text.split('\t').enumerate() {
            if i > 0 {
                let col = (line_start + offset) % terminal_width;
                // col < terminal_width <= u16::MAX
                let tab_width = tab_expansion_width(col as u16, self.terminal_size.width());
                offset += usize::from(tab_width);
            }
            offset += segment.width();
        }

        self.mark_cursor_position(offset as isize);
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

    /// Marks the cursor position right after `text`, as if `text` were
    /// written from the current position. Unlike a plain width offset, this
    /// accounts for how tabs are expanded when rendered.
    pub fn mark_cursor_position_after(&mut self, text: &str) {
        match &mut self.state {
            RenderState::Rendered(_) | RenderState::Initial => {}
            RenderState::ActiveRender { current_frame, .. } => {
                current_frame.mark_cursor_position_after(text);
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

    pub fn finish_current_frame(&mut self, add_empty_line: bool) -> io::Result<()> {
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

        if add_empty_line {
            self.terminal.write("\n")?;
            self.cursor_position.row += 1;
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
            .unwrap_or_default()
            .unwrap_or_default();

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

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use crate::{
        error::InquireResult,
        terminal::{test::MockTerminal, TerminalSize},
    };

    use super::FrameRenderer;

    #[test]
    fn ensure_inline_ansi_codes_are_maintained() -> InquireResult<()> {
        let mut output = VecDeque::new();
        let terminal = MockTerminal::new(&mut output).with_size(TerminalSize::default());
        let mut renderer = FrameRenderer::new(terminal)?;

        renderer.start_frame()?;
        renderer.write("Hello")?;
        renderer.write("World")?;
        renderer.write("\n")?;
        renderer.write("\x1b[1;31mWhat\x1b[0m is your name?")?;
        renderer.finish_current_frame(false)?;

        let terminal = &mut renderer.terminal;

        terminal.match_text("Hello");
        terminal.match_text("World");
        terminal.match_text("\r");
        terminal.match_text("\n");
        terminal.match_text("\x1b[1;31mWhat\x1b[0m is your name?");

        Ok(())
    }

    #[test]
    fn cursor_position_multiline_overflow() -> InquireResult<()> {
        use super::RenderState;

        let mut output = VecDeque::new();
        // Set narrow terminal width to force text wrapping
        let terminal = MockTerminal::new(&mut output).with_size(TerminalSize::new(10, 10).unwrap());
        let mut renderer = FrameRenderer::new(terminal)?;

        renderer.start_frame()?;

        // Write exactly 10 characters to fill first line
        renderer.write("hello worl")?;

        // Test case 1: offset that spans multiple rows
        // Current line width = 10, offset = 15 means total = 25
        // This should put cursor at row=2, col=5 (25 / 10 = 2 extra rows, 25 % 10 = 5)
        renderer.mark_cursor_position(15);

        let expected_position = match &renderer.state {
            RenderState::ActiveRender { current_frame, .. } => {
                current_frame.expected_cursor_position
            }
            _ => None,
        };

        if let Some(pos) = expected_position {
            assert_eq!(
                pos.row, 2,
                "Expected cursor row to be 2 when spanning multiple lines"
            );
            assert_eq!(pos.col, 5, "Expected cursor col to be 5 after wrapping");
        } else {
            panic!("Expected cursor position to be set");
        }

        // Test case 2: exact multiple of terminal width
        // offset = 20 means total = 30, should be row=3, col=0
        renderer.mark_cursor_position(20);

        let expected_position = match &renderer.state {
            RenderState::ActiveRender { current_frame, .. } => {
                current_frame.expected_cursor_position
            }
            _ => None,
        };

        if let Some(pos) = expected_position {
            assert_eq!(pos.row, 3, "Expected cursor row to be 3 for exact multiple");
            assert_eq!(pos.col, 0, "Expected cursor col to be 0 for exact multiple");
        } else {
            panic!("Expected cursor position to be set");
        }

        // Test case 3: spanning multiple lines
        // offset = 99 means total = 109, should be row=10, col=9
        renderer.mark_cursor_position(99);

        let expected_position = match &renderer.state {
            RenderState::ActiveRender { current_frame, .. } => {
                current_frame.expected_cursor_position
            }
            _ => None,
        };

        if let Some(pos) = expected_position {
            assert_eq!(
                pos.row, 10,
                "Expected cursor row to be 3 for exact multiple"
            );
            assert_eq!(pos.col, 9, "Expected cursor col to be 0 for exact multiple");
        } else {
            panic!("Expected cursor position to be set");
        }

        Ok(())
    }

    mod tabs {
        use std::collections::VecDeque;

        use super::super::{FrameRenderer, FrameState, RenderState};
        use crate::{
            error::InquireResult,
            terminal::{
                test::{emulate_screen, MockTerminal, MockTerminalToken},
                TerminalSize,
            },
            ui::{Color, Styled},
        };

        fn frame_with_width(width: u16) -> FrameState {
            FrameState::new(TerminalSize::new(width, 24).unwrap())
        }

        fn rows(frame: &FrameState) -> Vec<String> {
            frame
                .finished_rows
                .iter()
                .map(|row| {
                    row.get_content()
                        .iter()
                        .map(|styled| styled.content.as_str())
                        .collect()
                })
                .collect()
        }

        fn render(width: u16, text: &str) -> Vec<String> {
            let mut frame = frame_with_width(width);
            frame.write(&Styled::new(text));
            frame.finish();
            rows(&frame)
        }

        #[test]
        fn tab_at_line_start_expands_to_full_tab_stop() {
            assert_eq!(vec!["        x"], render(80, "\tx"));
        }

        #[test]
        fn tab_mid_stop_expands_to_next_tab_stop() {
            assert_eq!(vec!["abc     x"], render(80, "abc\tx"));
            assert_eq!(vec!["abcdefghi       x"], render(80, "abcdefghi\tx"));
        }

        #[test]
        fn tab_exactly_on_a_stop_expands_to_the_following_stop() {
            assert_eq!(vec!["abcdefgh        x"], render(80, "abcdefgh\tx"));
        }

        #[test]
        fn consecutive_tabs_expand_to_consecutive_stops() {
            assert_eq!(vec!["a       \u{20}       b"], render(80, "a\t\tb"));
        }

        #[test]
        fn tab_width_is_counted_in_line_width() {
            let mut frame = frame_with_width(80);
            frame.write(&Styled::new("ab\t"));
            assert_eq!(8, frame.current_line_width);
            frame.write(&Styled::new("c"));
            assert_eq!(9, frame.current_line_width);
            frame.finish();
            assert_eq!(9, frame.frame_size.width());
        }

        #[test]
        fn tab_column_includes_previously_written_fragments() {
            let mut frame = frame_with_width(80);
            frame.write(&Styled::new("> "));
            frame.write(&Styled::new("ab\tc"));
            frame.finish();
            assert_eq!(vec!["> ab    c"], rows(&frame));
        }

        #[test]
        fn tab_in_styled_fragment_keeps_style() {
            let mut frame = frame_with_width(80);
            frame.write(&Styled::new("ab"));
            frame.write(&Styled::new("c\td").with_fg(Color::LightRed));
            frame.finish();

            let row = frame.finished_rows[0].get_content();
            assert_eq!(2, row.len());
            assert_eq!("ab", row[0].content);
            assert_eq!("c     d", row[1].content);
            assert_eq!(Some(Color::LightRed), row[1].style.fg);
        }

        #[test]
        fn tab_after_wide_chars_uses_their_display_width() {
            // each CJK char and the emoji are two columns wide
            assert_eq!(vec!["日本    x"], render(80, "日本\tx"));
            assert_eq!(vec!["🍞      x"], render(80, "🍞\tx"));
            assert_eq!(vec!["a日本   x"], render(80, "a日本\tx"));
        }

        #[test]
        fn tab_near_right_edge_stops_at_the_edge() {
            // tab at col 8 of a 10-column row: only 2 columns left
            assert_eq!(vec!["abcdefgh  ", "x"], render(10, "abcdefgh\tx"));
            assert_eq!(vec!["abcdefghi ", "x"], render(10, "abcdefghi\tx"));
        }

        #[test]
        fn tab_on_full_row_starts_the_next_row() {
            assert_eq!(vec!["abcdefghij", "        x"], render(10, "abcdefghij\tx"));
        }

        #[test]
        fn tab_stops_restart_on_wrapped_rows() {
            // the tab is at column 1 of the second physical row
            assert_eq!(
                vec!["abcdefghij", "k       x"],
                render(10, "abcdefghijk\tx")
            );
        }

        #[test]
        fn tab_on_terminal_narrower_than_a_tab_stop() {
            assert_eq!(vec!["ab   ", "x"], render(5, "ab\tx"));
        }

        fn cursor_after(width: u16, before: &str, text: &str) -> (u16, u16) {
            let mut frame = frame_with_width(width);
            frame.write(&Styled::new(before));
            frame.mark_cursor_position_after(text);
            let cursor = frame.expected_cursor_position.unwrap();
            (cursor.row, cursor.col)
        }

        #[test]
        fn cursor_after_text_without_tabs_matches_width_offset() {
            assert_eq!((0, 7), cursor_after(80, "> ", "abc日"));
            assert_eq!((1, 1), cursor_after(10, "> ", "abcdefghi"));
        }

        #[test]
        fn cursor_after_text_with_tabs_matches_rendered_columns() {
            // "> ab" ends at col 4, tab advances to 8, then "c"
            assert_eq!((0, 9), cursor_after(80, "> ", "ab\tc"));
            assert_eq!((0, 8), cursor_after(80, "> ", "ab\t"));
            assert_eq!((0, 12), cursor_after(80, "日本", "\t日本"));
        }

        #[test]
        fn cursor_after_tab_matches_rendered_wrapping() {
            // check the cursor against what is actually rendered for the same text
            let cases = [
                (10, "abcdefgh", "\tx"),
                (10, "abcdefghij", "\tx"),
                (10, "abcdefghi", "jk\tx"),
                (10, "", "abcdefghijk\t"),
            ];

            for (width, before, text) in cases {
                let mut frame = frame_with_width(width);
                frame.write(&Styled::new(before));
                frame.mark_cursor_position_after(text);
                let cursor = frame.expected_cursor_position.unwrap();
                let cursor = (cursor.row, cursor.col);

                frame.write(&Styled::new(text));
                let expected = (frame.finished_rows.len() as u16, frame.current_line_width);
                let expected = if expected.1 == width {
                    (expected.0 + 1, 0)
                } else {
                    expected
                };

                assert_eq!(
                    expected, cursor,
                    "{:?} + {:?} at width {}",
                    before, text, width
                );
            }
        }

        fn render_frames(
            width: u16,
            frames: &[&str],
        ) -> InquireResult<VecDeque<MockTerminalToken>> {
            let mut output = VecDeque::new();
            {
                let terminal =
                    MockTerminal::new(&mut output).with_size(TerminalSize::new(width, 24).unwrap());
                let mut renderer = FrameRenderer::new(terminal)?;
                for frame in frames {
                    renderer.start_frame()?;
                    renderer.write(frame)?;
                    renderer.finish_current_frame(false)?;
                }
            }
            Ok(output)
        }

        #[test]
        fn tabs_are_never_sent_to_the_terminal() -> InquireResult<()> {
            let output = render_frames(20, &["a\tb\n\tc", "x\ty"])?;
            for token in &output {
                if let MockTerminalToken::Text(text) = token {
                    assert!(!text.content.contains('\t'), "{:?}", token);
                }
            }
            Ok(())
        }

        #[test]
        fn rendering_same_frame_twice_writes_nothing_new() -> InquireResult<()> {
            fn written_text(output: &VecDeque<MockTerminalToken>) -> Vec<&str> {
                output
                    .iter()
                    .filter_map(|token| match token {
                        MockTerminalToken::Text(text) if !text.content.trim().is_empty() => {
                            Some(text.content.as_str())
                        }
                        _ => None,
                    })
                    .collect()
            }

            let frame = "> a\tb\n  long option\twith a tab that wraps";
            let once = render_frames(20, &[frame])?;
            let twice = render_frames(20, &[frame, frame])?;

            assert_eq!(written_text(&once), written_text(&twice));
            Ok(())
        }

        #[test]
        fn redraws_with_tabs_match_space_only_control() -> InquireResult<()> {
            let width = 30;
            let size = TerminalSize::new(width, 24).unwrap();
            // the same content, with tabs and with the spaces they expand to
            let tabbed = [
                "? Pick\n> [ ] alice\tadmin\tsome long description\n  [ ] bob\tviewer\n",
                "? Pick\n  [ ] alice\tadmin\tsome long description\n> [ ] bob\tviewer\n",
            ];
            let spaced = [
                "? Pick\n> [ ] alice     admin   some long description\n  [ ] bob       viewer\n",
                "? Pick\n  [ ] alice     admin   some long description\n> [ ] bob       viewer\n",
            ];

            let sequence = [0, 1, 0, 1, 0, 1, 0];
            let tabbed: Vec<&str> = sequence.iter().map(|i| tabbed[*i]).collect();
            let spaced: Vec<&str> = sequence.iter().map(|i| spaced[*i]).collect();

            let tabbed_screens = emulate_screen(&render_frames(width, &tabbed)?, size);
            let spaced_screens = emulate_screen(&render_frames(width, &spaced)?, size);

            assert_eq!(spaced_screens, tabbed_screens);
            // Redraws of the same frame look the same. The first render is not
            // compared because redrawing a row that fills the whole width clears
            // its last cell, with or without tabs.
            assert_eq!(tabbed_screens[2], tabbed_screens[4]);
            assert_eq!(tabbed_screens[2], tabbed_screens[6]);
            Ok(())
        }

        #[test]
        fn renderer_state_is_rendered_after_finish() -> InquireResult<()> {
            let mut output = VecDeque::new();
            let terminal = MockTerminal::new(&mut output);
            let mut renderer = FrameRenderer::new(terminal)?;
            renderer.start_frame()?;
            renderer.write("a\tb")?;
            renderer.finish_current_frame(false)?;
            match &renderer.state {
                RenderState::Rendered(frame) => assert_eq!(vec!["a       b"], rows(frame)),
                _ => panic!("expected a rendered frame"),
            }
            Ok(())
        }
    }
}
