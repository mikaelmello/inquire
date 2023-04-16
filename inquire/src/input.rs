use unicode_segmentation::UnicodeSegmentation;

use crate::ui::{InnerAction, Key, KeyModifiers};

#[derive(Clone, Debug)]
pub struct Input {
    content: String,
    placeholder: Option<String>,
    cursor: usize,
    length: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Magnitude {
    Char,
    Word,
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineDirection {
    Left,
    Right,
}

fn is_alphanumeric(grapheme: &str) -> bool {
    grapheme.unicode_words().count() > 0
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputAction {
    Delete(Magnitude, LineDirection),
    MoveCursor(Magnitude, LineDirection),
    Write(char),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputHandleResult {
    ContentChanged,
    PositionChanged,
    Clean,
}

impl InputHandleResult {
    pub fn needs_redraw(&self) -> bool {
        match self {
            InputHandleResult::ContentChanged => true,
            InputHandleResult::PositionChanged => true,
            InputHandleResult::Clean => false,
        }
    }
}

impl InputAction {
    fn gen_write_from_str(value: &str) -> Vec<InputAction> {
        value
            .chars()
            .into_iter()
            .map(|c| InputAction::Write(c))
            .collect()
    }
}

impl InnerAction<()> for InputAction {
    fn from_key(key: Key, _config: &()) -> Option<Self>
    where
        Self: Sized,
    {
        let action = match key {
            Key::Backspace => Self::Delete(Magnitude::Char, LineDirection::Left),
            Key::Char('h', m) if m.contains(KeyModifiers::CONTROL) => {
                // Ctrl+Backspace is tricky, we don't want to handle a Ctrl+H
                // but also don't want ctrl+h to simply write h.
                // Let's catch this combination and ignore it.
                return None;
            }

            Key::Delete(m) if m.contains(KeyModifiers::CONTROL) => {
                Self::Delete(Magnitude::Word, LineDirection::Right)
            }
            Key::Delete(_) => Self::Delete(Magnitude::Char, LineDirection::Right),

            Key::Home => Self::MoveCursor(Magnitude::Line, LineDirection::Left),
            Key::Left(m) if m.contains(KeyModifiers::CONTROL) => {
                Self::MoveCursor(Magnitude::Word, LineDirection::Left)
            }
            Key::Left(_) => Self::MoveCursor(Magnitude::Char, LineDirection::Left),

            Key::End => Self::MoveCursor(Magnitude::Line, LineDirection::Right),
            Key::Right(m) if m.contains(KeyModifiers::CONTROL) => {
                Self::MoveCursor(Magnitude::Word, LineDirection::Right)
            }
            Key::Right(_) => Self::MoveCursor(Magnitude::Char, LineDirection::Right),

            Key::Char(c, _) => Self::Write(c),
            _ => return None,
        };

        Some(action)
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            placeholder: None,
            cursor: 0,
            length: 0,
        }
    }

    pub fn new_with<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        let content: String = content.into();

        let len = content.graphemes(true).count();

        Self {
            content,
            placeholder: None,
            length: len,
            cursor: len,
        }
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(String::from(placeholder));
        self
    }

    pub fn with_cursor(mut self, cursor: usize) -> Self {
        assert!(
            cursor <= self.length,
            "cursor index {} should be less than or equal to content length {}",
            cursor,
            self.length,
        );
        self.cursor = cursor;

        self
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn handle(&mut self, action: InputAction) -> InputHandleResult {
        match action {
            InputAction::MoveCursor(mag, dir) => match dir {
                LineDirection::Left => self.move_left(mag),
                LineDirection::Right => self.move_right(mag),
            },
            InputAction::Delete(mag, dir) => match dir {
                LineDirection::Left => self.backwards_delete(mag),
                LineDirection::Right => self.forwards_delete(mag),
            },
            InputAction::Write(c) => self.insert(c),
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
        self.length = 0;
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn pre_cursor(&self) -> &str {
        if self.cursor == self.length {
            &self.content[..]
        } else {
            let last = self.content[..]
                .grapheme_indices(true)
                .take(self.cursor.saturating_add(1))
                .last();

            match last {
                Some((beg, _)) => &self.content[..beg],
                None => &self.content[..],
            }
        }
    }

    fn move_left(&mut self, mag: Magnitude) -> InputHandleResult {
        if self.cursor == 0 {
            return InputHandleResult::Clean;
        }

        match mag {
            Magnitude::Char => self.cursor = self.cursor.saturating_sub(1),
            Magnitude::Word => self.cursor = self.prev_word_index(),
            Magnitude::Line => self.cursor = 0,
        }

        InputHandleResult::PositionChanged
    }

    fn move_right(&mut self, mag: Magnitude) -> InputHandleResult {
        match self.cursor.cmp(&self.length) {
            std::cmp::Ordering::Equal => InputHandleResult::Clean,
            std::cmp::Ordering::Less => {
                match mag {
                    Magnitude::Char => self.cursor = self.cursor.saturating_add(1),
                    Magnitude::Word => self.cursor = self.next_word_index(),
                    Magnitude::Line => self.cursor = self.length,
                }

                InputHandleResult::PositionChanged
            }
            std::cmp::Ordering::Greater => {
                self.cursor = self.length;
                InputHandleResult::PositionChanged
            }
        }
    }

    fn next_word_index(&mut self) -> usize {
        let graphemes = self.content.graphemes(true).enumerate().skip(self.cursor);
        let mut seen_word = false;

        for (idx, g) in graphemes {
            if is_alphanumeric(g) {
                seen_word = true;
            } else if seen_word {
                return idx;
            }
        }

        self.length
    }

    fn prev_word_index(&mut self) -> usize {
        let mut seen_word = false;
        let left = self.cursor;
        let right = self.length - left;
        let graphemes = self
            .content
            .graphemes(true)
            .rev()
            .skip(right)
            .enumerate()
            .map(|(idx, g)| (idx.saturating_add(1), g)); // Item.0 = distance to cursor

        for (dist, g) in graphemes {
            if is_alphanumeric(g) {
                seen_word = true;
            } else if seen_word {
                // word found
                return self.cursor.saturating_sub(dist - 1);
            }
        }

        0
    }

    fn insert(&mut self, c: char) -> InputHandleResult {
        let at = self.cursor;

        if at >= self.length {
            self.content.push(c);
            if self.update_length() {
                self.cursor = self.cursor.saturating_add(1);
            }
            return InputHandleResult::ContentChanged;
        }

        let mut result = String::new();
        for (index, grapheme) in self.content[..].graphemes(true).enumerate() {
            if index == at {
                result.push(c);
            }
            result.push_str(grapheme);
        }

        self.content = result;
        if self.update_length() {
            self.cursor = self.cursor.saturating_add(1);
        }

        InputHandleResult::ContentChanged
    }

    fn backwards_delete(&mut self, mag: Magnitude) -> InputHandleResult {
        if self.cursor == 0 {
            return InputHandleResult::Clean;
        }

        let cur_cursor_pos = self.cursor;
        let new_cursor_pos = match mag {
            Magnitude::Char => self.cursor.saturating_sub(1),
            Magnitude::Word => self.prev_word_index(),
            Magnitude::Line => 0,
        };

        if new_cursor_pos == cur_cursor_pos {
            return InputHandleResult::Clean;
        }

        self.cursor = new_cursor_pos;
        self.delete_chars_at_right(cur_cursor_pos - new_cursor_pos)
    }

    fn forwards_delete(&mut self, mag: Magnitude) -> InputHandleResult {
        let start = self.cursor;
        let end = match mag {
            Magnitude::Char => start.saturating_add(1),
            Magnitude::Word => self.next_word_index(),
            Magnitude::Line => self.length(),
        };

        let len = end - start;

        self.delete_chars_at_right(len)
    }

    fn delete_chars_at_right(&mut self, qty: usize) -> InputHandleResult {
        let start = self.cursor;
        let end = start.saturating_add(qty);

        let mut new_content: String = String::new();
        let mut length = 0;
        let mut result = InputHandleResult::Clean;

        for (index, grapheme) in self.content[..].graphemes(true).enumerate() {
            if index < start || index >= end {
                length += 1;
                new_content.push_str(grapheme);
            } else {
                result = InputHandleResult::ContentChanged;
            }
        }

        self.length = length;
        self.content = new_content;

        result
    }

    fn update_length(&mut self) -> bool {
        let new_len = self.content[..].graphemes(true).count();
        let old_len = self.length;
        self.length = new_len;

        new_len != old_len
    }
}

#[cfg(test)]
mod test {
    use unicode_segmentation::UnicodeSegmentation;

    use super::Input;
    use crate::input::{InputAction, InputHandleResult, LineDirection, Magnitude};

    #[test]
    fn move_previous_word() {
        let content = "great 🌍, 🍞, 🚗, 1231321📞, 🎉, 🍆xsa232 s2da ake iak eaik";

        let assert = |expected, initial| {
            let mut input = Input::new_with(content).with_cursor(initial);

            let result = input.handle(InputAction::MoveCursor(
                Magnitude::Word,
                LineDirection::Left,
            ));
            let cursor_moved = result == InputHandleResult::ContentChanged
                || result == InputHandleResult::PositionChanged;

            assert_eq!(expected != initial, cursor_moved,
                "cursor_moved '{}' is not equal to expected '{}' because of initial and expected cursors '{}' and '{}'",
                cursor_moved, expected != initial, initial, expected);
            assert_eq!(
                expected,
                input.cursor(),
                "unexpected result cursor from initial {initial}",
            );
        };

        for i in 0..16 {
            assert(0, i);
        }
        for i in 16..30 {
            assert(15, i);
        }
        for i in 30..37 {
            assert(29, i);
        }
        for i in 37..42 {
            assert(36, i);
        }
        for i in 42..46 {
            assert(41, i);
        }
        for i in 46..50 {
            assert(45, i);
        }
        for i in 50..54 {
            assert(49, i);
        }
    }

    #[test]
    // https://github.com/mikaelmello/inquire/issues/5
    fn regression_issue_5() {
        let heart = '♥';
        let vs16 = '\u{fe0f}';

        let heart_without_vs16 = String::from(heart);
        let heart_with_vs16 = "♥️";
        // let heart_with_vs16_char = '♥️'; // this doesn't compile because there are 2 chars
        let built_heart_with_vs16 = {
            let mut s = String::from(heart);
            s.push(vs16);
            s
        };

        assert_eq!(6, heart_with_vs16.len());
        assert_eq!(2, heart_with_vs16.chars().count());
        assert_eq!(1, heart_with_vs16.graphemes(true).count());
        assert_eq!(&built_heart_with_vs16, heart_with_vs16);

        let mut input = Input::new();
        assert_eq!(0, input.length);
        assert_eq!(0, input.cursor);
        assert_eq!("", input.content);

        input.insert(heart);
        {
            assert_eq!(1, input.length);
            assert_eq!(1, input.cursor);
            assert_eq!(heart_without_vs16, input.content);
            assert_ne!(heart_with_vs16, input.content);
            assert!(input.content.find(vs16).is_none());
        }

        input.insert(vs16);
        {
            assert_eq!(1, input.length);
            assert_eq!(1, input.cursor);
            assert_ne!(heart_without_vs16, input.content);
            assert_eq!(heart_with_vs16, input.content);
            assert!(input.content.find(vs16).is_some());
        }
    }
}
