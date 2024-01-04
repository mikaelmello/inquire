use crate::error::InquireResult;

use super::Key;

pub trait InputReader: Sized {
    fn read_key(&mut self) -> InquireResult<Key>;
}
