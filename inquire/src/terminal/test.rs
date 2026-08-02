#![cfg_attr(feature = "testing", allow(dead_code))]

use std::{collections::VecDeque, fmt::Display};

#[cfg(feature = "testing")]
pub use crate::ui::{Key, KeyModifiers};

use crate::ui::Styled;

use super::{Terminal, TerminalSize};

pub struct MockTerminal<'a> {
    pub size: TerminalSize,
    pub output: &'a mut VecDeque<MockTerminalToken>,
}

/// A terminal operation captured while rendering a prompt.
///
/// # Example
/// ```rust
/// use inquire::testing::MockTerminalToken;
///
/// let token = MockTerminalToken::from("hello");
/// assert_eq!(format!("{token:?}").contains("hello"), true);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockTerminalToken {
    /// Text written to the terminal (with styles).
    Text(Styled<String>),
    /// Clear the current line.
    ClearLine,
    /// Clear from cursor until the end of the line.
    ClearUntilNewLine,
    /// Hide the cursor.
    CursorHide,
    /// Show the cursor.
    CursorShow,
    /// Move the cursor up by N rows.
    CursorUp(u16),
    /// Move the cursor down by N rows.
    CursorDown(u16),
    /// Move the cursor left by N columns.
    CursorLeft(u16),
    /// Move the cursor right by N columns.
    CursorRight(u16),
    #[allow(dead_code)]
    /// Move the cursor to the given column.
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

// -----------------------------------------------------------------------------
// Public, feature-gated testing backend
// -----------------------------------------------------------------------------

#[cfg(feature = "testing")]
mod scripted {
    use std::{cell::RefCell, collections::VecDeque, fmt::Display, io::Result, io::Write, rc::Rc};

    use super::MockTerminalToken;

    use crate::{
        error::{InquireError, InquireResult},
        terminal::{Terminal, TerminalSize},
        testing::Key,
        ui::Styled,
    };

    /// A single rendered frame.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), inquire::InquireError> {
    /// use inquire::{Select, testing::{with_input, Key}};
    ///
    /// let (_result, report) = with_input(vec![Key::Enter], || {
    ///     Select::new("Pick one", vec!["A", "B"]).prompt()
    /// });
    ///
    /// let frame0 = &report.trace().frames()[0];
    /// assert_eq!(frame0.tokens().is_empty(), false);
    /// # Ok(()) }
    /// ```
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct MockTerminalFrame {
        tokens: Vec<MockTerminalToken>,
    }

    impl MockTerminalFrame {
        /// Returns the raw terminal tokens for this frame.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let frame0 = &report.trace().frames()[0];
        /// assert_eq!(frame0.tokens().len() > 0, true);
        /// # Ok(()) }
        /// ```
        pub fn tokens(&self) -> &[MockTerminalToken] {
            &self.tokens
        }

        /// Dumps this frame as a token-per-line debug string.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// // Ensure deterministic rendering (no ANSI color styling).
        /// std::env::set_var("NO_COLOR", "1");
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let dump = report.trace().frames()[0].to_token_dump_string();
        /// let expected = r#"CursorHide
        /// Text(Styled { content: "?", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "Pick one", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: ">", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "A", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "B", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "[", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "↑↓ to move, enter to select, type to filter", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "]", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// CursorUp(3)
        /// CursorRight(11)
        /// CursorShow
        /// "#;
        /// assert_eq!(dump, expected);
        /// # Ok(()) }
        /// ```
        pub fn to_token_dump_string(&self) -> String {
            let mut out = String::new();
            for token in &self.tokens {
                out.push_str(&format!("{token:?}\n"));
            }
            out
        }

        /// Returns a best-effort concatenation of all text written in this frame.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// // Ensure deterministic rendering (no ANSI color styling).
        /// std::env::set_var("NO_COLOR", "1");
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let frame0 = &report.trace().frames()[0];
        /// let expected = "? Pick one  \r\n> A\r\n  B\r\n[↑↓ to move, enter to select, type to filter]\r";
        /// assert_eq!(frame0.best_effort_text(), expected);
        /// # Ok(()) }
        /// ```
        pub fn best_effort_text(&self) -> String {
            let mut out = String::new();
            for token in &self.tokens {
                if let MockTerminalToken::Text(styled) = token {
                    out.push_str(&styled.content);
                }
            }
            out
        }

        /// Returns true if this frame contains `str` in its concatenated text output.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let frame0 = &report.trace().frames()[0];
        /// assert_eq!(frame0.contains_text("Pick one"), true);
        /// # Ok(()) }
        /// ```
        pub fn contains_text(&self, str: &str) -> bool {
            self.best_effort_text().contains(str)
        }
    }

    /// Captured output from a prompt, grouped by flushes (frames).
    /// Note: Intentionally no "slideshow" helper here: use `to_screen_string_up_to(...)`
    /// in a loop if you want per-frame snapshots.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), inquire::InquireError> {
    /// use inquire::{Select, testing::{with_input, Key}};
    ///
    /// let (_result, report) = with_input(vec![Key::Enter], || {
    ///     Select::new("Pick one", vec!["A", "B"]).prompt()
    /// });
    ///
    /// let trace = report.trace();
    /// assert_eq!(trace.frames().len() > 0, true);
    /// # Ok(()) }
    /// ```
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct MockTerminalTrace {
        frames: Vec<MockTerminalFrame>,
    }

    impl MockTerminalTrace {
        /// Returns the captured frames.
        pub fn frames(&self) -> &[MockTerminalFrame] {
            &self.frames
        }

        /// Returns true if any frame contains `needle` in its concatenated text output.
        pub fn contains_text(&self, str: &str) -> bool {
            self.frames.iter().any(|f| f.contains_text(str))
        }

        /// Dumps this trace as a frame-by-frame token dump.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// // Ensure deterministic rendering (no ANSI color styling).
        /// std::env::set_var("NO_COLOR", "1");
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let dump = report.trace().to_token_dump_string();
        /// let expected = r#"== Frame 0 ==
        /// CursorHide
        /// Text(Styled { content: "?", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "Pick one", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: ">", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "A", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "B", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "[", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "↑↓ to move, enter to select, type to filter", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "]", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// CursorUp(3)
        /// CursorRight(11)
        /// CursorShow
        ///
        /// == Frame 1 ==
        /// CursorHide
        /// CursorLeft(11)
        /// Text(Styled { content: "?", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "Pick one", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: " ", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "A", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// ClearUntilNewLine
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// ClearLine
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// ClearLine
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// ClearLine
        /// Text(Styled { content: "\r", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// Text(Styled { content: "\n", style: StyleSheet { fg: None, bg: None, att: Attributes(0x0) } })
        /// CursorShow
        ///
        /// == Frame 2 ==
        /// CursorUp(3)
        /// CursorShow
        ///
        /// "#;
        /// assert_eq!(dump, expected);
        /// # Ok(()) }
        /// ```
        pub fn to_token_dump_string(&self) -> String {
            let mut out = String::new();
            for (idx, frame) in self.frames.iter().enumerate() {
                out.push_str(&format!("== Frame {idx} ==\n"));
                out.push_str(&frame.to_token_dump_string());
                out.push('\n');
            }
            out
        }

        /// Pretty-prints the trace as a frame-by-frame token dump.
        ///
        /// Note: this is a debug-oriented view of captured terminal operations, not a
        /// human-friendly recreation of the prompt UI.
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let pretty = report.trace().to_pretty_string();
        /// let expected = report.trace().to_token_dump_string();
        /// assert_eq!(pretty, expected);
        /// # Ok(()) }
        /// ```
        pub fn to_pretty_string(&self) -> String {
            self.to_token_dump_string()
        }

        /// Replays all captured frames into an ANSI escape sequence stream.
        ///
        /// This is meant for *visual* debugging: printing the resulting string to a terminal
        /// can recreate what `inquire` drew, since it includes cursor movement and clear ops.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// // Ensure deterministic rendering (no ANSI color styling).
        /// std::env::set_var("NO_COLOR", "1");
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let ansi = report.trace().to_ansi_string();
        /// let expected = concat!(
        ///     "\x1b[?25l? Pick one  \r\n> A\r\n  B\r\n[↑↓ to move, enter to select, type to filter]\r",
        ///     "\x1b[3A\x1b[11C\x1b[?25h\x1b[?25l\x1b[11D? Pick one A\x1b[0K\r\n\x1b[2K\r\n\x1b[2K\r\n\x1b[2K\r\n\x1b[?25h\x1b[3A\x1b[?25h",
        /// );
        /// assert_eq!(ansi, expected);
        /// # Ok(()) }
        /// ```
        pub fn to_ansi_string(&self) -> String {
            if self.frames.is_empty() {
                return String::new();
            }
            self.to_ansi_string_up_to(self.frames.len() - 1)
        }

        /// Replays frames `0..=frame_idx` into an ANSI escape sequence stream.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// // Ensure deterministic rendering (no ANSI color styling).
        /// std::env::set_var("NO_COLOR", "1");
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let first = report.trace().to_ansi_string_up_to(0);
        /// let expected = concat!(
        ///     "\x1b[?25l? Pick one  \r\n> A\r\n  B\r\n[↑↓ to move, enter to select, type to filter]\r",
        ///     "\x1b[3A\x1b[11C\x1b[?25h",
        /// );
        /// assert_eq!(first, expected);
        /// # Ok(()) }
        /// ```
        pub fn to_ansi_string_up_to(&self, frame_idx: usize) -> String {
            let mut out: Vec<u8> = Vec::new();
            let _ = self.write_ansi_up_to(&mut out, frame_idx);
            String::from_utf8_lossy(&out).to_string()
        }

        /// Writes an ANSI stream for all frames to `writer`.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let mut buf = Vec::new();
        /// report.trace().write_ansi(&mut buf)?;
        /// assert_eq!(buf.is_empty(), false);
        /// # Ok(()) }
        /// ```
        pub fn write_ansi<W: Write>(&self, writer: &mut W) -> Result<()> {
            if self.frames.is_empty() {
                return Ok(());
            }
            self.write_ansi_up_to(writer, self.frames.len() - 1)
        }

        /// Writes an ANSI stream for frames `0..=frame_idx` to `writer`.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::with_input, ui::{Key, KeyModifiers}};
        ///
        /// let (_result, report) = with_input(vec![Key::Down(KeyModifiers::NONE), Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B", "C"]).prompt()
        /// });
        ///
        /// let mut buf = Vec::new();
        /// report.trace().write_ansi_up_to(&mut buf, 0)?;
        /// let s = String::from_utf8_lossy(&buf);
        /// assert_eq!(s.contains("Pick one"), true);
        /// # Ok(()) }
        /// ```
        pub fn write_ansi_up_to<W: Write>(&self, writer: &mut W, frame_idx: usize) -> Result<()> {
            if self.frames.is_empty() {
                return Ok(());
            }
            let max_idx = frame_idx.min(self.frames.len().saturating_sub(1));
            for frame in &self.frames[..=max_idx] {
                frame.write_ansi(writer)?;
            }
            Ok(())
        }

        /// Creates a *plain-text* snapshot of the final rendered screen.
        ///
        /// This interprets captured [`MockTerminalToken`]s (cursor movement, clears, etc.) into a
        /// best-effort 2D text buffer and returns the resulting visible contents.
        ///
        /// Unlike [`Trace::to_ansi_string`], this output contains no escape sequences and is
        /// suitable for assertions and documentation.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// // Ensure deterministic rendering (no ANSI color styling).
        /// std::env::set_var("NO_COLOR", "1");
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// assert_eq!(report.trace().to_screen_string(), "? Pick one A");
        /// # Ok(()) }
        /// ```
        pub fn to_screen_string(&self) -> String {
            if self.frames.is_empty() {
                return String::new();
            }
            self.to_screen_string_up_to(self.frames.len() - 1)
        }

        /// Creates a *plain-text* snapshot of the rendered screen after replaying frames
        /// `0..=frame_idx`.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// // Ensure deterministic rendering (no ANSI color styling).
        /// std::env::set_var("NO_COLOR", "1");
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let frame0 = report.trace().to_screen_string_up_to(0);
        /// let expected = "? Pick one\n> A\n  B\n[↑↓ to move, enter to select, type to filter]";
        /// assert_eq!(frame0, expected);
        /// # Ok(()) }
        /// ```
        pub fn to_screen_string_up_to(&self, frame_idx: usize) -> String {
            if self.frames.is_empty() {
                return String::new();
            }

            let mut screen = Screen::default();
            let max_idx = frame_idx.min(self.frames.len().saturating_sub(1));
            for frame in &self.frames[..=max_idx] {
                screen.apply_frame(frame);
            }
            screen.render()
        }
    }

    #[derive(Debug, Default, Clone)]
    struct Screen {
        lines: Vec<Vec<char>>,
        x: usize,
        y: usize,
    }

    impl Screen {
        fn apply_frame(&mut self, frame: &MockTerminalFrame) {
            for token in &frame.tokens {
                self.apply_token(token);
            }
        }

        fn apply_token(&mut self, token: &MockTerminalToken) {
            match token {
                MockTerminalToken::Text(styled) => {
                    for ch in styled.content.chars() {
                        match ch {
                            '\r' => self.x = 0,
                            '\n' => {
                                self.y = self.y.saturating_add(1);
                                self.x = 0;
                            }
                            _ => self.put_char(ch),
                        }
                    }
                }
                MockTerminalToken::ClearLine => self.clear_line(),
                MockTerminalToken::ClearUntilNewLine => self.clear_until_eol(),
                MockTerminalToken::CursorHide | MockTerminalToken::CursorShow => {}
                MockTerminalToken::CursorUp(n) => self.y = self.y.saturating_sub(*n as usize),
                MockTerminalToken::CursorDown(n) => self.y = self.y.saturating_add(*n as usize),
                MockTerminalToken::CursorLeft(n) => self.x = self.x.saturating_sub(*n as usize),
                MockTerminalToken::CursorRight(n) => self.x = self.x.saturating_add(*n as usize),
                MockTerminalToken::CursorMoveToColumn(col) => self.x = *col as usize,
            }
        }

        fn ensure_line(&mut self) {
            if self.y >= self.lines.len() {
                self.lines.resize_with(self.y + 1, Vec::new);
            }
        }

        fn ensure_col(&mut self) {
            self.ensure_line();
            let line = &mut self.lines[self.y];
            if self.x > line.len() {
                line.resize(self.x, ' ');
            }
        }

        fn put_char(&mut self, ch: char) {
            self.ensure_col();
            let line = &mut self.lines[self.y];
            if self.x == line.len() {
                line.push(ch);
            } else {
                line[self.x] = ch;
            }
            self.x = self.x.saturating_add(1);
        }

        fn clear_line(&mut self) {
            self.ensure_line();
            self.lines[self.y].clear();
        }

        fn clear_until_eol(&mut self) {
            self.ensure_line();
            let line = &mut self.lines[self.y];
            if self.x < line.len() {
                line.truncate(self.x);
            }
        }

        fn render(&self) -> String {
            if self.lines.is_empty() {
                return String::new();
            }

            let mut rendered: Vec<String> = self
                .lines
                .iter()
                .map(|line| {
                    let s: String = line.iter().collect();
                    s.trim_end_matches(' ').to_string()
                })
                .collect();

            while rendered.last().is_some_and(|l| l.is_empty()) {
                rendered.pop();
            }

            rendered.join("\n")
        }
    }

    impl MockTerminalFrame {
        /// Writes this frame's terminal operations as ANSI escape sequences.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// let frame0 = &report.trace().frames()[0];
        /// let mut buf = Vec::new();
        /// frame0.write_ansi(&mut buf)?;
        /// assert_eq!(buf.is_empty(), false);
        /// # Ok(()) }
        /// ```
        pub fn write_ansi<W: Write>(&self, writer: &mut W) -> Result<()> {
            for token in &self.tokens {
                write_terminal_token_ansi(writer, token)?;
            }
            Ok(())
        }
    }

    fn write_terminal_token_ansi<W: Write>(
        writer: &mut W,
        token: &MockTerminalToken,
    ) -> Result<()> {
        match token {
            MockTerminalToken::Text(styled) => {
                write_style_prefix(writer, styled.style)?;
                write!(writer, "{}", styled.content)?;
                write_style_suffix(writer, styled.style)?;
            }
            MockTerminalToken::ClearLine => write!(writer, "\x1b[2K")?,
            MockTerminalToken::ClearUntilNewLine => write!(writer, "\x1b[0K")?,
            MockTerminalToken::CursorHide => write!(writer, "\x1b[?25l")?,
            MockTerminalToken::CursorShow => write!(writer, "\x1b[?25h")?,
            MockTerminalToken::CursorUp(n) => {
                if *n > 0 {
                    write!(writer, "\x1b[{}A", n)?;
                }
            }
            MockTerminalToken::CursorDown(n) => {
                if *n > 0 {
                    write!(writer, "\x1b[{}B", n)?;
                }
            }
            MockTerminalToken::CursorRight(n) => {
                if *n > 0 {
                    write!(writer, "\x1b[{}C", n)?;
                }
            }
            MockTerminalToken::CursorLeft(n) => {
                if *n > 0 {
                    write!(writer, "\x1b[{}D", n)?;
                }
            }
            MockTerminalToken::CursorMoveToColumn(idx) => {
                // ANSI `G` is 1-based.
                write!(writer, "\x1b[{}G", (*idx as u32) + 1)?;
            }
        }
        Ok(())
    }

    fn write_style_prefix<W: Write>(writer: &mut W, style: crate::ui::StyleSheet) -> Result<()> {
        if let Some(fg) = style.fg {
            write_color(writer, fg, false)?;
        }
        if let Some(bg) = style.bg {
            write_color(writer, bg, true)?;
        }
        if style.att.contains(crate::ui::Attributes::BOLD) {
            write!(writer, "\x1b[1m")?;
        }
        if style.att.contains(crate::ui::Attributes::ITALIC) {
            write!(writer, "\x1b[3m")?;
        }
        Ok(())
    }

    fn write_style_suffix<W: Write>(writer: &mut W, style: crate::ui::StyleSheet) -> Result<()> {
        if style.fg.is_some() {
            write!(writer, "\x1b[39m")?;
        }
        if style.bg.is_some() {
            write!(writer, "\x1b[49m")?;
        }
        if !style.att.is_empty() {
            // SGR reset (matches crossterm's `SetAttribute(Attribute::Reset)` behavior).
            write!(writer, "\x1b[0m")?;
        }
        Ok(())
    }

    fn write_color<W: Write>(writer: &mut W, color: crate::ui::Color, is_bg: bool) -> Result<()> {
        use crate::ui::Color;

        let (prefix_normal, prefix_bright) = if is_bg { (40u8, 100u8) } else { (30u8, 90u8) };

        match color {
            Color::Black => write!(writer, "\x1b[{}m", prefix_normal + 0)?,
            Color::DarkRed => write!(writer, "\x1b[{}m", prefix_normal + 1)?,
            Color::DarkGreen => write!(writer, "\x1b[{}m", prefix_normal + 2)?,
            Color::DarkYellow => write!(writer, "\x1b[{}m", prefix_normal + 3)?,
            Color::DarkBlue => write!(writer, "\x1b[{}m", prefix_normal + 4)?,
            Color::DarkMagenta => write!(writer, "\x1b[{}m", prefix_normal + 5)?,
            Color::DarkCyan => write!(writer, "\x1b[{}m", prefix_normal + 6)?,
            Color::Grey => write!(writer, "\x1b[{}m", prefix_normal + 7)?,
            Color::DarkGrey => write!(writer, "\x1b[{}m", prefix_bright + 0)?,
            Color::LightRed => write!(writer, "\x1b[{}m", prefix_bright + 1)?,
            Color::LightGreen => write!(writer, "\x1b[{}m", prefix_bright + 2)?,
            Color::LightYellow => write!(writer, "\x1b[{}m", prefix_bright + 3)?,
            Color::LightBlue => write!(writer, "\x1b[{}m", prefix_bright + 4)?,
            Color::LightMagenta => write!(writer, "\x1b[{}m", prefix_bright + 5)?,
            Color::LightCyan => write!(writer, "\x1b[{}m", prefix_bright + 6)?,
            Color::White => write!(writer, "\x1b[{}m", prefix_bright + 7)?,
            Color::Rgb { r, g, b } => {
                if is_bg {
                    write!(writer, "\x1b[48;2;{};{};{}m", r, g, b)?;
                } else {
                    write!(writer, "\x1b[38;2;{};{};{}m", r, g, b)?;
                }
            }
            Color::AnsiValue(n) => {
                if is_bg {
                    write!(writer, "\x1b[48;5;{}m", n)?;
                } else {
                    write!(writer, "\x1b[38;5;{}m", n)?;
                }
            }
        }
        Ok(())
    }

    /// Report returned by [`with_input`].
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), inquire::InquireError> {
    /// use inquire::{Select, testing::{with_input, Key}};
    ///
    /// let (_result, report) = with_input(vec![Key::Enter], || {
    ///     Select::new("Pick one", vec!["A", "B"]).prompt()
    /// });
    ///
    /// assert_eq!(report.trace().frames().is_empty(), false);
    /// # Ok(()) }
    /// ```
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct TestReport {
        trace: MockTerminalTrace,
        consumed_input: Vec<Key>,
        remaining_input: Vec<Key>,
    }

    impl TestReport {
        /// Captured output from the prompt(s), grouped by frames.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// assert_eq!(report.trace().contains_text("Pick one"), true);
        /// # Ok(()) }
        /// ```
        pub fn trace(&self) -> &MockTerminalTrace {
            &self.trace
        }

        /// Keys that were consumed by prompts while the session was active.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{ui::{Key, KeyModifiers}, Select, testing::with_input};
        ///
        /// let (_result, report) = with_input(vec![Key::Down(KeyModifiers::NONE), Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B", "C"]).prompt()
        /// });
        ///
        /// assert_eq!(report.consumed_input(), &[Key::Down(KeyModifiers::NONE), Key::Enter]);
        /// # Ok(()) }
        /// ```
        pub fn consumed_input(&self) -> &[Key] {
            &self.consumed_input
        }

        /// Keys that were not consumed by any prompt.
        ///
        /// # Example
        /// ```rust
        /// use inquire::{testing::with_input, ui::{Key, KeyModifiers}};
        ///
        /// let (_result, report) = with_input(
        ///     vec![Key::Enter, Key::Down(KeyModifiers::NONE)],
        ///     || Ok::<_, inquire::InquireError>(()),
        /// );
        ///
        /// assert_eq!(report.remaining_input(), &[Key::Enter, Key::Down(KeyModifiers::NONE)]);
        /// ```
        pub fn remaining_input(&self) -> &[Key] {
            &self.remaining_input
        }

        /// Convenience helper to assert that all scripted input was consumed.
        ///
        /// # Example
        /// ```rust
        /// # fn main() -> Result<(), inquire::InquireError> {
        /// use inquire::{Select, testing::{with_input, Key}};
        ///
        /// let (_result, report) = with_input(vec![Key::Enter], || {
        ///     Select::new("Pick one", vec!["A", "B"]).prompt()
        /// });
        ///
        /// report.assert_all_input_consumed();
        /// assert_eq!(report.remaining_input().is_empty(), true);
        /// # Ok(()) }
        /// ```
        pub fn assert_all_input_consumed(&self) {
            assert!(
                self.remaining_input.is_empty(),
                "Unconsumed scripted input: {:?}",
                self.remaining_input
            );
        }
    }

    #[derive(Debug, Default)]
    struct SessionState {
        keys: VecDeque<Key>,
        consumed: Vec<Key>,
        frames: Vec<MockTerminalFrame>,
        cur_frame: MockTerminalFrame,
        terminal_size: TerminalSize,
    }

    thread_local! {
        static SESSION: RefCell<Option<Rc<RefCell<SessionState>>>> = const { RefCell::new(None) };
    }

    /// Installs scripted input for the current thread for the duration of `f`.
    ///
    /// Returns the closure result along with a [`TestReport`] containing:
    /// - captured frames (`trace`),
    /// - consumed input keys, and
    /// - any remaining (unconsumed) input keys.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), inquire::InquireError> {
    /// use inquire::{Select, testing::{with_input, Key}};
    ///
    /// // Ensure deterministic rendering (no ANSI color styling).
    /// std::env::set_var("NO_COLOR", "1");
    ///
    /// let (result, report) = with_input(
    ///     vec![Key::Enter],
    ///     || Select::new("Pick one", vec!["A", "B"]).prompt(),
    /// );
    ///
    /// assert_eq!(result?, "A");
    /// report.assert_all_input_consumed();
    ///
    /// // Assert deterministic, plain-text snapshots.
    /// let expected_first_frame = "? Pick one\n> A\n  B\n[↑↓ to move, enter to select, type to filter]";
    /// assert_eq!(report.trace().to_screen_string_up_to(0), expected_first_frame);
    /// assert_eq!(report.trace().to_screen_string(), "? Pick one A");
    /// # Ok(()) }
    /// ```
    pub fn with_input<R>(keys: Vec<Key>, f: impl FnOnce() -> R) -> (R, TestReport) {
        let state = Rc::new(RefCell::new(SessionState {
            keys: keys.into(),
            consumed: Vec::new(),
            frames: Vec::new(),
            cur_frame: MockTerminalFrame::default(),
            terminal_size: TerminalSize::default(),
        }));

        let prev = SESSION.with(|cell| cell.replace(Some(state.clone())));
        let result = f();

        let report = {
            let mut st = state.borrow_mut();
            if !st.cur_frame.tokens.is_empty() {
                let frame = std::mem::take(&mut st.cur_frame);
                st.frames.push(frame);
            }
            TestReport {
                trace: MockTerminalTrace {
                    frames: st.frames.clone(),
                },
                consumed_input: st.consumed.clone(),
                remaining_input: st.keys.iter().copied().collect(),
            }
        };

        SESSION.with(|cell| {
            *cell.borrow_mut() = prev;
        });

        (result, report)
    }

    fn try_get_session() -> Option<Rc<RefCell<SessionState>>> {
        SESSION.with(|cell| cell.borrow().clone())
    }

    /// Internal input reader used by the testing backend.
    pub(crate) struct ScriptedKeyReader {
        state: Rc<RefCell<SessionState>>,
    }

    impl ScriptedKeyReader {
        fn new(state: Rc<RefCell<SessionState>>) -> Self {
            Self { state }
        }
    }

    impl crate::ui::InputReader for ScriptedKeyReader {
        fn read_key(&mut self) -> InquireResult<Key> {
            let mut st = self.state.borrow_mut();
            match st.keys.pop_front() {
                Some(k) => {
                    st.consumed.push(k);
                    Ok(k)
                }
                None => {
                    let trace = MockTerminalTrace {
                        frames: {
                            let mut frames = st.frames.clone();
                            if !st.cur_frame.tokens.is_empty() {
                                frames.push(st.cur_frame.clone());
                            }
                            frames
                        },
                    };
                    Err(InquireError::TestInputExhausted { trace })
                }
            }
        }
    }

    /// Internal terminal implementation used by the testing backend.
    pub(crate) struct CaptureTerminal {
        state: Rc<RefCell<SessionState>>,
    }

    impl CaptureTerminal {
        fn new(state: Rc<RefCell<SessionState>>) -> Self {
            Self { state }
        }

        fn push(&mut self, token: MockTerminalToken) {
            self.state.borrow_mut().cur_frame.tokens.push(token);
        }
    }

    impl Terminal for CaptureTerminal {
        fn get_size(&self) -> Result<Option<TerminalSize>> {
            Ok(Some(self.state.borrow().terminal_size))
        }

        fn write<T: Display>(&mut self, val: T) -> Result<()> {
            self.push(val.into());
            Ok(())
        }

        fn write_styled<T: Display>(&mut self, val: &Styled<T>) -> Result<()> {
            self.push(
                Styled::new(format!("{}", val.content))
                    .with_style_sheet(val.style)
                    .into(),
            );
            Ok(())
        }

        fn clear_line(&mut self) -> Result<()> {
            self.push(MockTerminalToken::ClearLine);
            Ok(())
        }

        fn clear_until_new_line(&mut self) -> Result<()> {
            self.push(MockTerminalToken::ClearUntilNewLine);
            Ok(())
        }

        fn cursor_hide(&mut self) -> Result<()> {
            self.push(MockTerminalToken::CursorHide);
            Ok(())
        }

        fn cursor_show(&mut self) -> Result<()> {
            self.push(MockTerminalToken::CursorShow);
            Ok(())
        }

        fn cursor_up(&mut self, cnt: u16) -> Result<()> {
            self.push(MockTerminalToken::CursorUp(cnt));
            Ok(())
        }

        fn cursor_down(&mut self, cnt: u16) -> Result<()> {
            self.push(MockTerminalToken::CursorDown(cnt));
            Ok(())
        }

        fn cursor_left(&mut self, cnt: u16) -> Result<()> {
            self.push(MockTerminalToken::CursorLeft(cnt));
            Ok(())
        }

        fn cursor_right(&mut self, cnt: u16) -> Result<()> {
            self.push(MockTerminalToken::CursorRight(cnt));
            Ok(())
        }

        fn cursor_move_to_column(&mut self, idx: u16) -> Result<()> {
            self.push(MockTerminalToken::CursorMoveToColumn(idx));
            Ok(())
        }

        fn flush(&mut self) -> Result<()> {
            let mut st = self.state.borrow_mut();
            let frame = std::mem::take(&mut st.cur_frame);
            st.frames.push(frame);
            Ok(())
        }
    }

    /// Returns a headless `(InputReader, Terminal)` pair that is backed by the
    /// current thread's scripted testing session.
    pub(crate) fn get_default_terminal_for_testing(
    ) -> InquireResult<(ScriptedKeyReader, CaptureTerminal)> {
        let state = try_get_session().ok_or(InquireError::TestingNotInitialized)?;
        Ok((
            ScriptedKeyReader::new(state.clone()),
            CaptureTerminal::new(state),
        ))
    }
}

#[cfg(feature = "testing")]
pub use scripted::{with_input, MockTerminalFrame, MockTerminalTrace, TestReport};

#[cfg(feature = "testing")]
pub(crate) use scripted::get_default_terminal_for_testing;
