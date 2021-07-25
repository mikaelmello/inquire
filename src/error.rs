//! Module containing definitions of inquire's error handling

use std::io;

use thiserror::Error;

/// Inquire errors
#[derive(Error, Debug)]
pub enum InquireError {
    /// The input device is not a TTY, which means that enabling raw mode
    /// on the terminal in order to listen to input events is not possible.
    #[error("The input device is not a TTY")]
    NotTTY,

    /// The given prompt configuration is not valid. A detailed error message
    /// is contained in the value string.
    #[error("The prompt configuration is invalid: {0}")]
    InvalidConfiguration(String),

    /// Error when executing IO operations.
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    /// The user canceled the operation by pressing Ctrl+C or ESC.
    #[error("Operation was canceled by the user")]
    OperationCanceled,
}

/// Result type where errors are from type InquireError
pub type InquireResult<T> = Result<T, InquireError>;
