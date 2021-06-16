#[macro_use]
extern crate simple_error;

mod config;
pub mod multiselect;
pub mod question;
mod renderer;
mod survey;
mod terminal;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
