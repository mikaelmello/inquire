use unicode_segmentation::UnicodeSegmentation;

use crate::ui::{Key, KeyModifiers};

#[derive(Clone, Debug)]
pub struct Input {
    content: String,
    placeholder: Option<String>,
    cursor: usize,
    length: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

    pub fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Backspace => self.backspace(),
            Key::Char('h', m) if m.contains(KeyModifiers::CONTROL) => false,

            Key::Delete(m) if m.contains(KeyModifiers::CONTROL) => self.delete_next_word(),
            Key::Delete(_) => self.delete(1),

            Key::Home => self.move_backward(MoveKind::Line),
            Key::Left(m) if m.contains(KeyModifiers::CONTROL) => self.move_backward(MoveKind::Word),
            Key::Left(_) => self.move_backward(MoveKind::Char),

            Key::End => self.move_forward(MoveKind::Line),
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

    fn move_backward(&mut self, kind: MoveKind) -> bool {
        if self.cursor == 0 {
            return false;
        }

        match kind {
            MoveKind::Char => self.cursor = self.cursor.saturating_sub(1),
            MoveKind::Word => self.cursor = self.prev_word_index(),
            MoveKind::Line => self.cursor = 0,
        }

        true
    }

    fn move_forward(&mut self, kind: MoveKind) -> bool {
        match self.cursor.cmp(&self.length) {
            std::cmp::Ordering::Equal => false,
            std::cmp::Ordering::Less => {
                match kind {
                    MoveKind::Char => self.cursor = self.cursor.saturating_add(1),
                    MoveKind::Word => self.cursor = self.next_word_index(),
                    MoveKind::Line => self.cursor = self.length,
                }

                true
            }
            std::cmp::Ordering::Greater => {
                self.cursor = self.length;
                true
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

    fn insert(&mut self, c: char) -> bool {
        let at = self.cursor;

        if at >= self.length {
            self.content.push(c);
            if self.update_length() {
                self.cursor = self.cursor.saturating_add(1);
            }
            return true;
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

        true
    }

    fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        self.cursor = self.cursor.saturating_sub(1);
        self.delete(1)
    }

    fn delete_next_word(&mut self) -> bool {
        let start = self.cursor;
        let end = self.next_word_index();

        let len = end - start;

        self.delete(len)
    }

    fn delete(&mut self, qty: usize) -> bool {
        let start = self.cursor;
        let end = start.saturating_add(qty);

        let mut result: String = String::new();
        let mut length = 0;
        let mut dirty = false;

        for (index, grapheme) in self.content[..].graphemes(true).enumerate() {
            if index < start || index >= end {
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
    use crate::ui::{Key, KeyModifiers};

    #[test]
    fn move_previous_word() {
        let content = "great 🌍, 🍞, 🚗, 1231321📞, 🎉, 🍆xsa232 s2da ake iak eaik";

        let assert = |expected, initial| {
            let mut input = Input::new_with(content).with_cursor(initial);

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
