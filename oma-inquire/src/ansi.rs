use std::{iter::Peekable, str::CharIndices};

#[must_use]
enum AnsiMatchResult {
    Matched { start: usize, end: usize },
    NotMatched,
}

fn matched(start: usize, end: usize) -> AnsiMatchResult {
    AnsiMatchResult::Matched { start, end }
}

/// Matches an ANSI escape code according to a simplified version of
/// [this description](https://vt100.net/emu/dec_ansi_parser). As we only want to know when the
/// automaton gets out of/reaches the "ground" state, we only keep track of transitions leading
/// out of it/into it.
///
/// The only way to get out of the ground is to read the escape character ('\x1b'). Transitions
/// like "anywhere -- \x9b -> csi_entry" are not supported, as few terminals implement them.
struct AnsiMatcher<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> AnsiMatcher<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
        }
    }

    #[inline]
    fn run(mut self) -> AnsiMatchResult {
        match self.chars.next() {
            Some((start_index, '\x1b')) => self.escape(start_index),
            _ => AnsiMatchResult::NotMatched,
        }
    }

    #[inline]
    fn next(&mut self) -> Option<u32> {
        self.chars.next().map(|(_, c)| c as u32)
    }

    #[inline]
    fn peek(&mut self) -> Option<(usize, u32)> {
        self.chars.peek().map(|(idx, c)| (*idx, *c as u32))
    }

    fn next_char_boundary(&mut self) -> usize {
        self.peek().map(|(idx, _)| idx).unwrap_or(self.input.len())
    }

    fn escape(mut self, start_index: usize) -> AnsiMatchResult {
        match self.next() {
            None => matched(start_index, self.input.len()),
            Some(0x5B) => self.csi_entry(start_index),
            Some(
                0x5D // osc_string
                | 0x50 // dcs_entry
                | 0x58 | 0x5E | 0x5F) => self.string(start_index), // sos/pm/apc_string
            Some(0x20..=0x2F) => self.escape_intermediate(start_index),
            Some(0x30..=0x4F | 0x51..=0x57 | 0x59 | 0x5A | 0x5C | 0x60..=0x7E) => {
                matched(start_index, self.next_char_boundary())
            }
            Some(0x1B | 0x7F | _) => self.escape(start_index),
        }
    }

    fn csi_entry(mut self, start_index: usize) -> AnsiMatchResult {
        match self.next() {
            Some(0x1B) => self.escape(start_index),
            Some(0x40..=0x7E) => matched(start_index, self.next_char_boundary()),
            None => matched(start_index, self.input.len()),
            _ => self.csi_entry(start_index), // loop until match
        }
    }

    fn escape_intermediate(mut self, start_index: usize) -> AnsiMatchResult {
        match self.next() {
            Some(0x1B) => self.escape(start_index),
            Some(0x30..=0x7E) => matched(start_index, self.next_char_boundary()),
            None => matched(start_index, self.input.len()),
            _ => self.escape_intermediate(start_index), // loop until match
        }
    }

    /// Matches until the end of sos/pm/apc strings, dcs entries and osc strings.
    fn string(mut self, start_index: usize) -> AnsiMatchResult {
        match self.next() {
            Some(0x1B) => self.escape(start_index),
            Some(0x07 | 0x9C) => matched(start_index, self.next_char_boundary()),
            None => matched(start_index, self.input.len()),
            _ => self.string(start_index), // loop until match
        }
    }
}

/// An iterator that is aware of ANSI escape codes.
pub struct AnsiAwareChars<'a> {
    pub input: &'a str,
}

#[must_use]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AnsiAwareChar<'a> {
    AnsiEscapeSequence(&'a str),
    Char(char),
}

impl<'a> Iterator for AnsiAwareChars<'a> {
    type Item = AnsiAwareChar<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match AnsiMatcher::new(self.input).run() {
            AnsiMatchResult::Matched { start, end } => {
                let matched_slice = &self.input[start..end];
                self.input = &self.input[end..];
                Some(AnsiAwareChar::AnsiEscapeSequence(matched_slice))
            }
            AnsiMatchResult::NotMatched => {
                let mut chars = self.input.char_indices();
                match chars.next() {
                    Some((idx, c)) => {
                        self.input = &self.input[idx + c.len_utf8()..];
                        Some(AnsiAwareChar::Char(c))
                    }
                    None => None,
                }
            }
        }
    }
}

/// An iterator that strips ANSI escape codes from a string.
///
/// Often constructed by calling [`ansi_stripped_chars`].
pub struct AnsiStrippedChars<'a> {
    pub input: &'a str,
}

impl<'a> Iterator for AnsiStrippedChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match AnsiMatcher::new(self.input).run() {
            AnsiMatchResult::Matched { end, .. } => {
                self.input = &self.input[end..];
                self.next()
            }
            AnsiMatchResult::NotMatched => {
                let mut chars = self.input.char_indices();
                match chars.next() {
                    Some((idx, c)) => {
                        self.input = &self.input[idx + c.len_utf8()..];
                        Some(c)
                    }
                    None => None,
                }
            }
        }
    }
}

/// Constructs an iterator over the chars of the input string, stripping away ANSI escape codes.
pub trait AnsiStrippable {
    #[allow(unused)]
    fn ansi_stripped_chars(&self) -> AnsiStrippedChars<'_>;
}

impl<T> AnsiStrippable for T
where
    T: AsRef<str>,
{
    fn ansi_stripped_chars(&self) -> AnsiStrippedChars<'_> {
        AnsiStrippedChars {
            input: self.as_ref(),
        }
    }
}

pub trait AnsiAware {
    fn ansi_aware_chars(&self) -> AnsiAwareChars<'_>;
}

impl<T> AnsiAware for T
where
    T: AsRef<str>,
{
    fn ansi_aware_chars(&self) -> AnsiAwareChars<'_> {
        AnsiAwareChars {
            input: self.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_stripped_eq {
        ($input:expr, $expected:expr) => {
            let stripped: String = $input.ansi_stripped_chars().collect();
            assert_eq!(&stripped, $expected);
        };
    }

    #[test]
    fn test_normal_ansi_escapes() {
        assert_stripped_eq!("1\x1b[0m2", "12");
        assert_stripped_eq!("\x1b[92mHello, \x1b[91mWorld!\x1b[0m", "Hello, World!");
        assert_stripped_eq!("\x1b[7@Hi", "Hi");
        assert_stripped_eq!(
            "\x1b]0;Set The Terminal Title To This\u{9c}Print This",
            "Print This"
        );
        assert_stripped_eq!("\x1b[96", "");
    }

    #[test]
    // Please don't use ansi escapes like these!
    fn test_inconsistencies() {
        // Some terminals will print "String\u{9c}Hello World", others will print nothing.
        assert_stripped_eq!("\x1b[\x1b]String\u{9c}Hello World", "Hello World");

        // Kitty will print "[38;5;43mABCD", but most will print "ABCD"
        assert_stripped_eq!("\x1b[\x1b[38;5;43mAB\x1b[48;5;10mCD\x1b[0m", "ABCD");

        // Kitty will print "[96mCat\n", but most will print "Cat\n"
        assert_stripped_eq!("\x1b\x19[96mCat\x1b[0m\n", "Cat\n");
    }

    #[test]
    fn ansi_aware_test_normal_ansi_escapes() {
        let chars: Vec<AnsiAwareChar<'_>> = "\x1b[92mHello, \x1b[91mWorld!\x1b[0m"
            .ansi_aware_chars()
            .collect();
        assert_eq!(
            chars,
            vec![
                AnsiAwareChar::AnsiEscapeSequence("\x1b[92m"),
                AnsiAwareChar::Char('H'),
                AnsiAwareChar::Char('e'),
                AnsiAwareChar::Char('l'),
                AnsiAwareChar::Char('l'),
                AnsiAwareChar::Char('o'),
                AnsiAwareChar::Char(','),
                AnsiAwareChar::Char(' '),
                AnsiAwareChar::AnsiEscapeSequence("\x1b[91m"),
                AnsiAwareChar::Char('W'),
                AnsiAwareChar::Char('o'),
                AnsiAwareChar::Char('r'),
                AnsiAwareChar::Char('l'),
                AnsiAwareChar::Char('d'),
                AnsiAwareChar::Char('!'),
                AnsiAwareChar::AnsiEscapeSequence("\x1b[0m"),
            ]
        );

        let chars: Vec<AnsiAwareChar<'_>> = "\x1b]0;Set The Terminal Title To This\u{9c}Print This"
            .ansi_aware_chars()
            .collect();
        assert_eq!(
            chars,
            vec![
                AnsiAwareChar::AnsiEscapeSequence("\x1b]0;Set The Terminal Title To This\u{9c}"),
                AnsiAwareChar::Char('P'),
                AnsiAwareChar::Char('r'),
                AnsiAwareChar::Char('i'),
                AnsiAwareChar::Char('n'),
                AnsiAwareChar::Char('t'),
                AnsiAwareChar::Char(' '),
                AnsiAwareChar::Char('T'),
                AnsiAwareChar::Char('h'),
                AnsiAwareChar::Char('i'),
                AnsiAwareChar::Char('s'),
            ]
        );
    }
}
