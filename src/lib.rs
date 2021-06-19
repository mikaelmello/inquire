#[macro_use]
extern crate simple_error;

pub mod ask;
pub mod config;
pub mod input;
pub mod multiselect;
pub mod question;
mod renderer;
pub mod select;
pub mod survey;
mod terminal;
mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
