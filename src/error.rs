//! Module containing definitions of inquire's error handling

use std::io;

use thiserror::Error;

/// Inquire errors
#[derive(Error, Debug)]
pub enum InquireError {
    /// The input device is not a TTY.
    #[error("The input device is not a TTY")]
    NotTTY,

    /// The input device is not a TTY.
    #[error("The prompt configuration is invalid: {0}")]
    InvalidConfiguration(String),

    /// Error when executing IO operations.
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    /// Invalid state when processing prompt.
    /// Created to check edge cases but should not happen.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Returned when the user canceled the operation, probably
    /// by pressing Ctrl+C or ESC.
    #[error("Operation was canceled by the user")]
    OperationCanceled,
}

/// Result type where errors are from type InquireError
pub type InquireResult<T> = Result<T, InquireError>;
