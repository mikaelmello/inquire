pub mod action;
#[cfg(test)]
mod test;

use unicode_segmentation::UnicodeSegmentation;

use crate::InputAction;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    content: String,
    placeholder: Option<String>,
    cursor: usize,
    length: usize,
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

    pub fn handle(&mut self, action: InputAction) -> InputActionResult {
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
            // hot path, skip counting graphemes every time.
            &self.content[..]
        } else {
            let last = self.content[..].grapheme_indices(true).nth(self.cursor);

            match last {
                Some((beg, _)) => &self.content[..beg],
                None => &self.content[..],
            }
        }
    }

    fn move_left(&mut self, mag: Magnitude) -> InputActionResult {
        if self.cursor == 0 {
            return InputActionResult::Clean;
        }

        match mag {
            Magnitude::Char => self.cursor = self.cursor.saturating_sub(1),
            Magnitude::Word => self.cursor = self.prev_word_index(),
            Magnitude::Line => self.cursor = 0,
        }

        InputActionResult::PositionChanged
    }

    fn move_right(&mut self, mag: Magnitude) -> InputActionResult {
        match self.cursor.cmp(&self.length) {
            std::cmp::Ordering::Equal => InputActionResult::Clean,
            std::cmp::Ordering::Less => {
                match mag {
                    Magnitude::Char => self.cursor = self.cursor.saturating_add(1),
                    Magnitude::Word => self.cursor = self.next_word_index(),
                    Magnitude::Line => self.cursor = self.length,
                }

                InputActionResult::PositionChanged
            }
            std::cmp::Ordering::Greater => {
                // this should never happen
                // but if there's a bug somewhere, we don't want to panic
                // so we just set the cursor to the end of the input
                self.cursor = self.length;
                InputActionResult::PositionChanged
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

    fn insert(&mut self, c: char) -> InputActionResult {
        let at = self.cursor;

        if at >= self.length {
            self.content.push(c);
            if self.update_length() {
                self.cursor = self.cursor.saturating_add(1);
            }
            return InputActionResult::ContentChanged;
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

        InputActionResult::ContentChanged
    }

    fn backwards_delete(&mut self, mag: Magnitude) -> InputActionResult {
        if self.cursor == 0 {
            return InputActionResult::Clean;
        }

        let cur_cursor_pos = self.cursor;
        let new_cursor_pos = match mag {
            Magnitude::Char => self.cursor.saturating_sub(1),
            Magnitude::Word => self.prev_word_index(),
            Magnitude::Line => 0,
        };

        if new_cursor_pos == cur_cursor_pos {
            return InputActionResult::Clean;
        }

        self.cursor = new_cursor_pos;
        self.delete_chars_at_right(cur_cursor_pos - new_cursor_pos)
    }

    fn forwards_delete(&mut self, mag: Magnitude) -> InputActionResult {
        let start = self.cursor;
        let end = match mag {
            Magnitude::Char => start.saturating_add(1),
            Magnitude::Word => self.next_word_index(),
            Magnitude::Line => self.length(),
        };

        let len = end - start;

        self.delete_chars_at_right(len)
    }

    fn delete_chars_at_right(&mut self, qty: usize) -> InputActionResult {
        let start = self.cursor;
        let end = start.saturating_add(qty);

        let mut new_content: String = String::new();
        let mut length = 0;
        let mut result = InputActionResult::Clean;

        for (index, grapheme) in self.content[..].graphemes(true).enumerate() {
            if index < start || index >= end {
                length += 1;
                new_content.push_str(grapheme);
            } else {
                result = InputActionResult::ContentChanged;
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputActionResult {
    ContentChanged,
    PositionChanged,
    Clean,
}

impl InputActionResult {
    pub fn needs_redraw(&self) -> bool {
        match self {
            InputActionResult::ContentChanged | InputActionResult::PositionChanged => true,
            InputActionResult::Clean => false,
        }
    }
}
