//! Definitions of `inquire`'s error handling

use std::io;

use thiserror::Error;

/// Type alias to define errors that might be thrown by the library user
/// on callbacks such as validators.
pub type CustomUserError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Possible errors returned by `inquire` prompts.
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

    /// Error while executing IO operations.
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    /// The user canceled the operation by pressing ESC.
    #[error("Operation was canceled by the user")]
    OperationCanceled,

    /// The operation was interrupted by the user after they
    /// pressed Ctrl+C.
    ///
    /// This error will be returned only when using `crossterm`
    /// or `termion` as the terminal back-end. If using `console`,
    /// pressing Ctrl+C will trigger SIGINT.
    #[error("Operation was interrupted by the user")]
    OperationInterrupted,

    /// Error while executing IO operations.
    #[error("User-provided error: {0}")]
    Custom(#[from] CustomUserError),
}

/// Result type where errors are of type [InquireError](crate::error::InquireError)
pub type InquireResult<T> = Result<T, InquireError>;
