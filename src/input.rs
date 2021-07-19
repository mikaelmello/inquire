use unicode_segmentation::UnicodeSegmentation;

use crate::key::{Key, KeyModifiers};

#[derive(Clone, Debug)]
pub struct Input {
    content: String,
    cursor: usize,
    length: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            length: 0,
        }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Backspace => self.backspace(),

            Key::Delete => self.delete(),

            Key::Home => self.move_left(true),
            Key::Left(m) => self.move_left(m.contains(KeyModifiers::CONTROL)),

            Key::End => self.move_right(true),
            Key::Right(m) => self.move_right(m.contains(KeyModifiers::CONTROL)),

            Key::Char(c, _) => self.insert(c),
            _ => false,
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
        self.length = 0;
    }

    // returns the substrings before, at and after the cursor
    pub fn split(&self) -> (String, String, String) {
        let mut before = String::new();
        let mut at = String::new();
        let mut after = String::new();

        for (index, grapheme) in self.content[..].graphemes(true).enumerate() {
            if index < self.cursor {
                before.push_str(grapheme);
            } else if index == self.cursor {
                at.push_str(grapheme);
            } else {
                after.push_str(grapheme);
            }
        }

        (before, at, after)
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    fn move_left(&mut self, until_start: bool) -> bool {
        if self.cursor == 0 {
            return false;
        }

        if until_start {
            self.cursor = 0;
        } else {
            self.cursor = self.cursor.saturating_sub(1);
        }

        true
    }

    fn move_right(&mut self, until_end: bool) -> bool {
        if self.cursor == self.length {
            return false;
        } else if self.cursor > self.length {
            // should never arrive here
            // but if it does, let's correct it.
            self.cursor = self.length;
            return true;
        }

        if until_end {
            self.cursor = self.length;
        } else {
            self.cursor = self.cursor.saturating_add(1);
        }

        true
    }

    fn insert(&mut self, c: char) -> bool {
        let at = self.cursor;
        self.cursor = self.cursor.saturating_add(1);

        if at >= self.length {
            self.content.push(c);
            self.length = self.length.saturating_add(1);
            return true;
        }

        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.content[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.length = length;
        self.content = result;

        true
    }

    fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        self.cursor = self.cursor.saturating_sub(1);
        self.delete()
    }

    fn delete(&mut self) -> bool {
        let at = self.cursor;

        let mut result: String = String::new();
        let mut length = 0;
        let mut dirty = false;

        for (index, grapheme) in self.content[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(grapheme);
            } else {
                dirty = true;
            }
        }

        self.length = length;
        self.content = result;

        dirty
    }
}
