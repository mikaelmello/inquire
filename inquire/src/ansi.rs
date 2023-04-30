use std::str::Chars;

#[must_use]
enum MatchResult<'a> {
    Matched { remaining_chars: Chars<'a> },
    NotMatched,
}

/// Match/consume the next ANSI escape code if it exists.
/// The format is based on [this description](https://handwiki.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences)
fn match_escape_code(chars: Chars<'_>) -> MatchResult<'_> {
    let mut original_chars_iterator = chars;
    let mut chars = original_chars_iterator.by_ref().peekable();

    macro_rules! should_match {
        ($c:expr, $pat:pat) => {
            #[allow(unused_parens)]
            match $c {
                Some($pat) => {}
                _ => return MatchResult::NotMatched,
            }
        };
    }

    // match \x1b[
    should_match!(chars.next(), '\x1b');
    should_match!(chars.next(), '[');

    // match [0–9:;<=>?]*
    while let Some('0'..='?') = chars.peek() {
        chars.next();
    }

    // match [ !"#$%&'()*+,-./]
    while let Some(' '..='/') = chars.peek() {
        chars.next();
    }

    // match [@A–Z[\]^_`a–z{|}~]
    should_match!(chars.next(), '@'..='~');

    MatchResult::Matched {
        remaining_chars: original_chars_iterator,
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
        match match_escape_code(chars) {
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
    fn strips_ansi_codes() {
        assert_stripped_eq!("1\x1b[0m2", "12");
        assert_stripped_eq!("\x1b[92mHello, \x1b[91mWorld!\x1b[0m", "Hello, World!");
        assert_stripped_eq!("\x1b[\x1b[38;5;43mAB\x1b[48;5;10mCD\x1b[0m", "\x1b[ABCD");
        assert_stripped_eq!("\x1b[7@Hi", "Hi");
    }
}
