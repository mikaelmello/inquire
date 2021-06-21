#[macro_use]
extern crate simple_error;

extern crate regex;

pub mod ask;
pub mod config;
pub mod prompts;
pub mod question;
mod renderer;
pub mod survey;
mod terminal;
mod utils;

pub use crate::prompts::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
