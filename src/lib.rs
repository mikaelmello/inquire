#[macro_use]
extern crate simple_error;

extern crate regex;

mod answer;
mod config;
mod formatter;
mod prompts;
mod renderer;
mod terminal;
mod utils;
pub mod validator;

pub use crate::prompts::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
