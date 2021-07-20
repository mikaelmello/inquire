use std::cmp::min;

use unicode_segmentation::UnicodeSegmentation;

use crate::key::{Key, KeyModifiers};

#[derive(Clone, Debug)]
pub struct Input {
    content: String,
    cursor: usize,
    length: usize,
}

enum MoveKind {
    Char,
    Word,
    Line,
}

fn is_alphanumeric(grapheme: &str) -> bool {
    grapheme.unicode_words().count() > 0
}

impl Input {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            length: 0,
        }
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = String::from(content);
        self.length = content.graphemes(true).count();
        self.cursor = min(self.cursor, self.length);

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

    pub fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Backspace => self.backspace(),

            Key::Delete => self.delete(),

            Key::Home => self.move_backward(MoveKind::Line),
            Key::Left(m) if m.contains(KeyModifiers::CONTROL) => self.move_backward(MoveKind::Word),
            Key::Left(_) => self.move_backward(MoveKind::Char),

            Key::End => self.move_forward(MoveKind::Char),
            Key::Right(m) if m.contains(KeyModifiers::CONTROL) => self.move_forward(MoveKind::Word),
            Key::Right(_) => self.move_forward(MoveKind::Char),

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

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn move_backward(&mut self, kind: MoveKind) -> bool {
        if self.cursor == 0 {
            return false;
        }

        match kind {
            MoveKind::Char => self.cursor = self.cursor.saturating_sub(1),
            MoveKind::Word => self.move_prev_word(),
            MoveKind::Line => self.cursor = 0,
        }

        true
    }

    fn move_forward(&mut self, kind: MoveKind) -> bool {
        if self.cursor == self.length {
            return false;
        } else if self.cursor > self.length {
            // should never arrive here
            // but if it does, let's correct it.
            self.cursor = self.length;
            return true;
        }

        match kind {
            MoveKind::Char => self.cursor = self.cursor.saturating_add(1),
            MoveKind::Word => self.move_next_word(),
            MoveKind::Line => self.cursor = self.length,
        }

        true
    }

    fn move_next_word(&mut self) {
        let graphemes = self.content.graphemes(true).enumerate().skip(self.cursor);
        let mut seen_word = false;

        for (idx, g) in graphemes {
            if is_alphanumeric(g) {
                seen_word = true;
            } else if seen_word {
                self.cursor = idx;
                return;
            }
        }

        self.cursor = self.length;
    }

    fn move_prev_word(&mut self) {
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
                self.cursor = self.cursor.saturating_sub(dist - 1);
                return;
            }
        }

        self.cursor = 0;
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

#[cfg(test)]
mod test {
    use super::Input;
    use crate::key::{Key, KeyModifiers};

    #[test]
    fn move_previous_word() {
        let content = "great 🌍, 🍞, 🚗, 1231321📞, 🎉, 🍆xsa232 s2da ake iak eaik";

        let assert = |expected, initial| {
            let mut input = Input::new().with_content(content).with_cursor(initial);

            let dirty = input.handle_key(Key::Left(KeyModifiers::CONTROL));
            assert_eq!(expected != initial, dirty,
                "dirty '{}' is not equal to expected '{}' because of initial and expected cursors '{}' and '{}'",
                dirty, expected != initial, initial, expected);
            assert_eq!(
                expected,
                input.cursor(),
                "unexpected result cursor from initial {}",
                initial
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
}
