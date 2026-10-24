//! `inquire` is a library for building interactive prompts on terminals.
//!
//! It provides several different prompts in order to interactively ask the user
//! for information via the CLI. With `inquire`, you can use:
//!
//! - [`Text`] to get text input from the user, with _built-in autocompletion support_;
//! - [`Editor`]* to get longer text inputs by opening a text editor for the user;
//! - [`DateSelect`]* to get a date input from the user, selected via an _interactive calendar_;
//! - [`Select`] to ask the user to select one option from a given list;
//! - [`MultiSelect`] to ask the user to select an arbitrary number of options from a given list;
//! - [`Confirm`] for simple yes/no confirmation prompts;
//! - [`CustomType`] for text prompts that you would like to parse to a custom type, such as numbers or UUIDs;
//! - [`Password`] for secretive text prompts.
//!
//! \* The Editor and DateSelect prompts are available by enabling the `editor` and `date` features, respectively.
//!
//! Check out the [GitHub repository](https://github.com/mikaelmello/inquire) to see demos of what you can do with `inquire`.
//!
//! # Features
//!
//! - Cross-platform, supporting UNIX and Windows terminals (thanks to [crossterm](https://crates.io/crates/crossterm));
//! - Several kinds of prompts to suit your needs;
//! - Support for fine-grained configuration for each prompt type, allowing you to customize:
//!   - Default values;
//!   - Input validators and formatters;
//!   - Help messages;
//!   - Autocompletion for [`Text`] prompts;
//!   - Custom list filters for Select and [`MultiSelect`] prompts;
//!   - Custom parsers for [`Confirm`] and [`CustomType`] prompts;
//!   - Custom extensions for files created by [`Editor`] prompts;
//!   - and many others!
//!
//! # Simple Example
//!
//! ```rust no_run
//! use inquire::{Text, validator::{StringValidator, Validation}};
//!
//! fn main() {
//!     let validator = |input: &str| if input.chars().count() > 140 {
//!         Ok(Validation::Invalid("You're only allowed 140 characters.".into()))
//!     } else {
//!         Ok(Validation::Valid)
//!     };
//!
//!     let status = Text::new("What are you thinking about?")
//!         .with_validator(validator)
//!         .prompt();
//!
//!     match status {
//!         Ok(status) => println!("Your status is being published..."),
//!         Err(err) => println!("Error while publishing your status: {}", err),
//!     }
//! }
//! ```
//!
//! # Terminal state on signals
//!
//! While a prompt is active, `inquire` changes the terminal state, and it restores
//! that state when the prompt finishes or is dropped. A signal such as `SIGTERM`
//! ends the process without running destructors, which can leave the terminal
//! without echo until the user runs `reset`. `inquire` does not install signal
//! handlers itself, since that is a process-wide decision for your application.
//! If your program can receive such signals while a prompt is open, register a
//! handler that restores the terminal before exiting, as shown in the
//! [`signal_cleanup.rs`](https://github.com/mikaelmello/inquire/blob/main/examples/signal_cleanup.rs)
//! example.
//!
//! The example uses the default `crossterm` back-end: it disables raw mode and
//! bracketed paste, and shows the cursor, which `inquire` hides briefly while
//! redrawing. The `termion` and `console` back-ends also use raw mode and hide
//! the cursor while redrawing, but don't enable bracketed paste, so the handler
//! only needs to restore the original terminal mode and show the cursor.
//!
//! [`Text`]: crate::Text
//! [`DateSelect`]: crate::DateSelect
//! [`Select`]: crate::Select
//! [`MultiSelect`]: crate::MultiSelect
//! [`Confirm`]: crate::Confirm
//! [`CustomType`]: crate::CustomType
//! [`Password`]: crate::Password
//! [`Editor`]: crate::Editor

#![warn(missing_docs)]
#![deny(unused_crate_dependencies)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::bool_to_int_with_if)]
mod ansi;
pub mod autocompletion;
mod config;
#[cfg(feature = "date")]
mod date_utils;
pub mod error;
pub mod formatter;
mod input;
pub mod list_option;
pub mod parser;
mod prompts;
mod terminal;
pub mod type_aliases;
pub mod ui;
mod utils;
pub mod validator;

pub use crate::autocompletion::Autocomplete;
pub use crate::config::set_global_render_config;
pub use crate::error::{CustomUserError, InquireError};
pub use crate::input::action::*;
pub use crate::prompts::*;
