use crate::error::InquireResult;

/// This function is a helpful one-liner to prompt the user for a date.
///
/// Under the hood, it is equivalent to calling `inquire::DateSelect::new(message).prompt()`.
/// See the documentation for [`inquire::DateSelect`] for more information on its behavior.
///
/// # Arguments
///
/// * `message`: A message that implements the `AsRef<str>` trait. This message will be displayed to the user
///   when asking for input.
///
/// # Returns
///
/// * `InquireResult<NaiveDate>`: An enum that represents the result of the prompt operation. If the operation is successful,
///  it returns `InquireResult::Ok(NaiveDate)` where NaiveDate's value is the date selected by the user. If the operation
/// encounters an error, it returns `InquireResult::Err(InquireError)`.
///
/// # Example
///
/// ``` no_run
/// # use inquire::{*, error::*};
/// let token = prompt_secret("Access Token:")?;
/// # inquire::error::InquireResult::Ok(())
/// ```
///
/// # Errors
///
/// This function will return an error if there is a problem interacting with the terminal, or if the user
/// cancels the operation by pressing `Ctrl+C`.
#[cfg(feature = "date")]
pub fn prompt_date<M>(message: M) -> InquireResult<chrono::NaiveDate>
where
    M: AsRef<str>,
{
    crate::DateSelect::new(message.as_ref()).prompt()
}
