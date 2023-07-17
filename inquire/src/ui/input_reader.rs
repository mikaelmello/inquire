use crate::error::InquireResult;

use super::Key;

pub trait InputReader {
    fn read_key(&mut self) -> InquireResult<Key>;
}
