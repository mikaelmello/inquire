#[macro_use]
extern crate simple_error;

extern crate regex;

pub mod answer;
mod ask;
pub mod config;
mod prompts;
mod renderer;
mod terminal;
mod utils;

pub use crate::prompts::*;
pub use answer::*;
pub use ask::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
