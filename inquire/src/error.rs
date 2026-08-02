//! Definitions of `inquire`'s error handling

use core::fmt;
use std::{error::Error, io};

/// Type alias to define errors that might be thrown by the library user
/// on callbacks such as validators.
pub type CustomUserError = Box<dyn Error + Send + Sync + 'static>;

/// Possible errors returned by `inquire` prompts.
#[derive(Debug)]
pub enum InquireError {
    /// The input device is not a TTY, which means that enabling raw mode
    /// on the terminal in order to listen to input events is not possible.
    NotTTY,

    /// The given prompt configuration is not valid. A detailed error message
    /// is contained in the value string.
    InvalidConfiguration(String),

    /// Error while executing IO operations.
    IO(io::Error),

    /// The user canceled the operation by pressing ESC.
    OperationCanceled,

    /// The operation was interrupted by the user after they
    /// pressed Ctrl+C.
    ///
    /// This error will be returned only when using `crossterm`
    /// or `termion` as the terminal back-end. If using `console`,
    /// pressing Ctrl+C will trigger SIGINT.
    OperationInterrupted,

    /// Error while executing IO operations.
    Custom(CustomUserError),

    /// The `testing` feature is enabled, but no scripted test input session is active.
    ///
    /// Use [`crate::testing::with_input`] to install scripted key input for the current
    /// thread before calling `.prompt()`.
    #[cfg(feature = "testing")]
    TestingNotInitialized,

    /// A prompt attempted to read another key, but the scripted input was exhausted.
    #[cfg(feature = "testing")]
    TestInputExhausted {
        /// Captured terminal frames up to the point of failure.
        trace: crate::testing::MockTerminalTrace,
    },
}

impl Error for InquireError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InquireError::IO(err) => Some(err),
            InquireError::Custom(err) => Some(err.as_dyn_error()),
            _ => None,
        }
    }
}

trait AsDynError<'a> {
    fn as_dyn_error(&self) -> &(dyn Error + 'a);
}

impl<'a> AsDynError<'a> for dyn Error + Send + Sync + 'a {
    #[inline]
    fn as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

impl From<CustomUserError> for InquireError {
    fn from(err: CustomUserError) -> Self {
        InquireError::Custom(err)
    }
}

impl From<io::Error> for InquireError {
    fn from(err: io::Error) -> Self {
        match err.raw_os_error() {
            Some(25 | 6) => InquireError::NotTTY,
            _ => InquireError::IO(err),
        }
    }
}

impl fmt::Display for InquireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InquireError::NotTTY => f.write_str("The input device is not a TTY"),
            InquireError::InvalidConfiguration(s) => {
                write!(f, "The prompt configuration is invalid: {}", s)
            }
            InquireError::IO(err) => write!(f, "IO error: {}", err),
            InquireError::OperationCanceled => f.write_str("Operation was canceled by the user"),
            InquireError::OperationInterrupted => {
                f.write_str("Operation was interrupted by the user")
            }
            InquireError::Custom(err) => write!(f, "User-provided error: {}", err),
            #[cfg(feature = "testing")]
            InquireError::TestingNotInitialized => f.write_str(
                "Testing backend not initialized. When the `testing` feature is enabled, \
                     `.prompt()` requires a scripted input session. Wrap your code with \
                     `inquire::testing::with_input(...)`.",
            ),
            #[cfg(feature = "testing")]
            InquireError::TestInputExhausted { trace } => {
                writeln!(
                    f,
                    "Scripted test input exhausted before the prompt completed."
                )?;
                write!(f, "\nCaptured frames:\n{}", trace.to_pretty_string())
            }
        }
    }
}

/// Result type where errors are of type [InquireError]
pub type InquireResult<T> = Result<T, InquireError>;
