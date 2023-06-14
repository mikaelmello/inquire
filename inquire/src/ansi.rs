use std::str::Chars;

#[must_use]
enum MatchResult<'a> {
    Matched { remaining_chars: Chars<'a> },
    NotMatched,
}

fn matched(chars: Chars<'_>) -> MatchResult<'_> {
    MatchResult::Matched {
        remaining_chars: chars,
    }
}

/// Matches an ANSI escape code according to a simplified version of
/// [this description](https://vt100.net/emu/dec_ansi_parser). As we only want to know when the
/// automaton gets out of/reaches the "ground" state, we only keep track of transitions leading
/// out of it/into it.
///
/// The only way to get out of the ground is to read the escape character ('\x1b'). Transitions
/// like "anywhere -- \x9b -> csi_entry" are not supported, as few terminals implement them.
struct Matcher<'a> {
    chars: Chars<'a>,
}

impl<'a> Matcher<'a> {
    fn new(chars: Chars<'a>) -> Self {
        Self { chars }
    }

    #[inline]
    fn run(mut self) -> MatchResult<'a> {
        match self.chars.next() {
            Some('\x1b') => self.escape(),
            _ => MatchResult::NotMatched,
        }
    }

    #[inline]
    fn next(&mut self) -> Option<u32> {
        self.chars.next().map(|c| c as u32)
    }

    fn escape(mut self) -> MatchResult<'a> {
        match self.next() {
            None => matched(self.chars),
            Some(0x5B) => self.csi_entry(),
            Some(
                0x5D // osc_string
                | 0x50 // dcs_entry
                | 0x58 | 0x5E | 0x5F) => self.string(), // sos/pm/apc_string
            Some(0x20..=0x2F) => self.escape_intermediate(),
            Some(0x30..=0x4F | 0x51..=0x57 | 0x59 | 0x5A | 0x5C | 0x60..=0x7E) => {
                matched(self.chars)
            }
            Some(0x1B | 0x7F | _) => self.escape(),
        }
    }

    fn csi_entry(mut self) -> MatchResult<'a> {
        match self.next() {
            Some(0x1B) => self.escape(),
            None | Some(0x40..=0x7E) => matched(self.chars),
            _ => self.csi_entry(), // loop until match
        }
    }

    fn escape_intermediate(mut self) -> MatchResult<'a> {
        match self.next() {
            Some(0x1B) => self.escape(),
            None | Some(0x30..=0x7E) => matched(self.chars),
            _ => self.escape_intermediate(), // loop until match
        }
    }

    /// Matches until the end of sos/pm/apc strings, dcs entries and osc strings.
    fn string(mut self) -> MatchResult<'a> {
        match self.next() {
            Some(0x1B) => self.escape(),
            None | Some(0x07 | 0x9C) => matched(self.chars),
            _ => self.string(), // loop until match
        }
    }
}

/// An iterator that strips ANSI escape codes from a string.
///
/// Often constructed by calling [`ansi_stripped_chars`].
pub struct AnsiStrippedChars<'a> {
    pub chars: Chars<'a>,
}

impl<'a> Iterator for AnsiStrippedChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let chars = self.chars.clone();
        match Matcher::new(chars).run() {
            MatchResult::Matched { remaining_chars } => {
                self.chars = remaining_chars;
                self.next()
            }
            MatchResult::NotMatched => self.chars.next(),
        }
    }
}

/// Constructs an iterator over the chars of the input string, stripping away ANSI escape codes.
pub trait AnsiStrippable {
    fn ansi_stripped_chars(&self) -> AnsiStrippedChars<'_>;
}

impl AnsiStrippable for &str {
    fn ansi_stripped_chars(&self) -> AnsiStrippedChars<'_> {
        AnsiStrippedChars {
            chars: self.chars(),
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
}
