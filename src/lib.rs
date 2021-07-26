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
//!   - Input a line of text
//!     - You can easily set-up auto-completion.
//!   - Confirm an action with yes or no responses
//!   - Input a text and have it automatically validated and parsed to retrieve any type
//!     - Useful when asking for numerical inputs, or even formatted ids!
//!   - Input a password
//!   - Pick a date from an interactive calendar.
//!
//! You can customize several aspects of each one of these prompts such as the page
//! size and the behavior of the filter when selecting options, help and error messages,
//! validate inputs, format the final output, etc.
//!
//! A more complete documentation is present in the `README.md` file of the repository.
//! Please go there while the proper docs are being updated.
//!
//! Example
//! ```rust no_run
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

pub mod config;
#[cfg(feature = "date")]
mod date_utils;
pub mod error;
pub mod formatter;
mod input;
mod key;
pub mod option_answer;
pub mod parser;
mod prompts;
mod renderer;
mod terminal;
mod utils;
pub mod validator;

pub use crate::prompts::*;
