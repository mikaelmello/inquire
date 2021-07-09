//! `inquire` is a library for building interactive prompts on terminals,
//! inspired by [survey](https://github.com/AlecAivazis/survey).
//!
//! It provides several different prompts in order to interactively ask the user
//! for information via the CLI.
//!
//! With `inquire`, you can ask the user to:
//!   - Select one option among many choices
//!   - Select many options among many choices
//!     - The user can filter the options on both selects!
//!   - Input a line of text (with possible suggestions)
//!   - Confirm an action with yes or no responses
//!   - Input a password without echoing the content
//!   - and more to come!
//!
//! You can customize several aspects of each one of these prompts such as the page
//! size and the behavior of the filter when selecting options, help and error messages,
//! validate inputs, format the final output, etc.
//!
//!
//! Example
//! ```rust
//! use inquire::{min_length, Text};
//!
//! fn main() {
//!     let name = Text::new("What is your name?")
//!         .with_validator(min_length!(8, "Sorry, this name is invalid"))
//!         .prompt();
//!     
//!     match name {
//!         Ok(name) => println!("Hello {}", name),
//!         Err(err) => println!("Error: {}", err),
//!     }
//! }
//! ```

#![warn(missing_docs)]

mod answer;
pub mod config;
#[cfg(feature = "date")]
mod date_utils;
pub mod error;
mod formatter;
mod parser;
mod prompts;
mod renderer;
mod terminal;
mod utils;
pub mod validator;

pub use crate::prompts::*;
pub use answer::OptionAnswer;
