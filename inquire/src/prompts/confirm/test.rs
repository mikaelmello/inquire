use std::vec;

use rstest::rstest;

use crate::{
    error::InquireResult,
    ui::{
        test::{FakeBackend, Token},
        Key, KeyModifiers,
    },
    Confirm, InquireError,
};

#[test]
fn prompt_can_be_initialized_from_str() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let result = Confirm::from("Question")
        .with_default(true)
        .prompt_with_backend(&mut backend)?;
    assert!(result, "Answer was not the expected one");

    Ok(())
}

#[rstest]
#[case("yes", true)]
#[case("y", true)]
#[case("YES", true)]
#[case("Y", true)]
#[case("no", false)]
#[case("n", false)]
#[case("NO", false)]
#[case("N", false)]
fn prompt_without_default_correctly_parses_input(
    #[case] input: &str,
    #[case] expected_result: bool,
) -> InquireResult<()> {
    let mut keys = Key::char_keys_from_str(input);
    keys.push(Key::Enter);

    let mut backend = FakeBackend::new(keys);

    let result = Confirm::from("Question")
        .with_default(true)
        .prompt_with_backend(&mut backend)?;
    assert_eq!(expected_result, result, "Answer was not the expected one");

    Ok(())
}

#[rstest]
fn escape_after_successful_submit_has_no_effect() -> InquireResult<()> {
    let mut keys = Key::char_keys_from_str("yes");
    keys.push(Key::Enter);
    keys.push(Key::Escape);

    let mut backend = FakeBackend::new(keys);

    let result = Confirm::from("Question")
        .with_default(true)
        .prompt_with_backend(&mut backend)?;
    assert!(result, "Answer was not the expected one");

    Ok(())
}

#[rstest]
#[case("yeah")]
#[case("si")]
#[case("1")]
#[case("nah")]
#[case("nn")]
#[case("0")]
fn invalid_inputs_are_properly_rejected(#[case] input: &str) -> InquireResult<()> {
    let mut keys = Key::char_keys_from_str(input);
    keys.push(Key::Enter);
    keys.push(Key::Escape);

    let mut backend = FakeBackend::new(keys);

    let result = Confirm::from("Question")
        .with_default(true)
        .prompt_with_backend(&mut backend);

    assert!(result.is_err(), "Result was not an error");
    assert!(
        matches!(result.unwrap_err(), InquireError::OperationCanceled),
        "Error message was not the expected one"
    );

    Ok(())
}

#[rstest]
#[case(true)]
#[case(false)]
fn prompt_with_default_can_be_readily_submitted(#[case] default_value: bool) -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let result = Confirm::from("Question")
        .with_default(default_value)
        .prompt_with_backend(&mut backend)?;
    assert_eq!(default_value, result, "Answer was not the expected one");

    Ok(())
}

#[rstest]
#[case("yes", true)]
#[case("no", false)]
fn prompt_with_valid_starting_input_can_be_readily_submitted(
    #[case] input: &str,
    #[case] expected_result: bool,
) -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let result = Confirm::from("Question")
        .with_starting_input(input)
        .prompt_with_backend(&mut backend)?;
    assert_eq!(expected_result, result, "Answer was not the expected one");

    Ok(())
}

#[rstest]
fn placeholder_is_rendered() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = Confirm::new("Question")
        .with_placeholder("Placeholder")
        .with_default(true)
        .prompt_with_backend(&mut backend)?;

    let rendered_frames = backend.frames();

    for (idx, frame) in rendered_frames.iter().enumerate() {
        let is_last_frame = idx == rendered_frames.len() - 1;

        if is_last_frame {
            assert!(
                frame.tokens().iter().all(|t| !matches!(t, Token::Input(_))),
                "Frame {} (last) contained an input token when it should not have",
                idx
            );
        } else {
            assert!(
                frame.tokens().iter().any(|t| matches!(t, Token::Input(input) if input.placeholder() == Some("Placeholder"))),
                "Frame {} did not contain a placeholder token",
                idx
            );
        }
    }

    Ok(())
}

#[rstest]
#[case("si", Some(true))]
#[case("no", Some(false))]
#[case("yes", None)]
#[case("nah", None)]
fn custom_parser_for_spanish_works_as_expected(
    #[case] input: &str,
    #[case] expected_result: Option<bool>,
) -> InquireResult<()> {
    let mut keys = Key::char_keys_from_str(input);
    keys.push(Key::Enter);
    keys.push(Key::Escape);

    let mut backend = FakeBackend::new(keys);

    let result = Confirm::new("Question")
        .with_parser(&|input| match input.to_lowercase().as_str() {
            "si" => Ok(true),
            "no" => Ok(false),
            _ => Err(()),
        })
        .prompt_with_backend(&mut backend);

    match (expected_result, result) {
        (Some(expected), Ok(result)) => {
            assert_eq!(expected, result, "Answer was not the expected one");
        }
        (None, Err(err)) => {
            assert!(
                matches!(err, InquireError::OperationCanceled),
                "Error was not the 'OperationCanceled' expected"
            );
        }
        (Some(expected_result), Err(_)) => {
            panic!("Result was not successful {} expected", expected_result);
        }
        (None, Ok(result)) => {
            panic!("Result was {} when a canceled prompt was expected", result);
        }
    }

    Ok(())
}

#[rstest]
fn default_error_message_is_rendered_on_invalid_input() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![
        Key::Char('c', KeyModifiers::NONE),
        Key::Enter,
        Key::Escape,
    ]);

    let result = Confirm::new("Question").prompt_with_backend(&mut backend);
    assert!(result.is_err(), "Result was not an error");
    assert!(
        matches!(result.unwrap_err(), InquireError::OperationCanceled),
        "Error message was not the expected one"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        4,
        rendered_frames.len(),
        "There should have been 4 frames rendered"
    );

    for (i, frame) in rendered_frames.iter().take(2).enumerate() {
        assert!(
            frame
                .tokens()
                .iter()
                .any(|t| !matches!(t, Token::ErrorMessage(_))),
            "Frame {} had an error message token when the two first frames should not have one",
            i
        );
    }

    assert!(
        rendered_frames[2].has_token(&Token::ErrorMessage(Confirm::DEFAULT_ERROR_MESSAGE.into())),
        "Third frame did not contain an error message token when one was expected",
    );

    Ok(())
}

#[rstest]
fn custom_error_message_is_rendered_on_invalid_input() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![
        Key::Char('c', KeyModifiers::NONE),
        Key::Enter,
        Key::Escape,
    ]);

    let result = Confirm::new("Question")
        .with_error_message("INCORRECT!!!!")
        .prompt_with_backend(&mut backend);

    assert!(result.is_err(), "Result was not an error");
    assert!(
        matches!(result.unwrap_err(), InquireError::OperationCanceled),
        "Error message was not the expected one"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        4,
        rendered_frames.len(),
        "There should have been 4 frames rendered"
    );

    for (i, frame) in rendered_frames.iter().take(2).enumerate() {
        assert!(
            frame
                .tokens()
                .iter()
                .any(|t| !matches!(t, Token::ErrorMessage(_))),
            "Frame {} had an error message token when the two first frames should not have one",
            i
        );
    }

    assert!(
        rendered_frames[2].has_token(&Token::ErrorMessage("INCORRECT!!!!".into())),
        "Third frame did not contain an error message token when one was expected",
    );

    Ok(())
}

#[rstest]
#[case(true, "Y/n")]
#[case(false, "y/N")]
fn default_formatter_for_default_values_follows_convention(
    #[case] default_value: bool,
    #[case] expected_output: &str,
) -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = Confirm::new("Question")
        .with_default(default_value)
        .prompt_with_backend(&mut backend)?;

    let rendered_frames = backend.frames();

    for (idx, frame) in rendered_frames.iter().enumerate() {
        let is_last_frame = idx == rendered_frames.len() - 1;

        if is_last_frame {
            assert!(
                frame
                    .tokens()
                    .iter()
                    .all(|t| !matches!(t, Token::DefaultValue(_))),
                "Frame {} (last) contained a help message token when it should not have",
                idx
            );
        } else {
            assert!(
                frame.has_token(&Token::DefaultValue(expected_output.into())),
                "Frame {} did not contain a help message token",
                idx
            );
        }
    }

    Ok(())
}

#[rstest]
#[case(true, "y")]
#[case(false, "n")]
fn custom_formatter_for_default_values_is_used(
    #[case] default_value: bool,
    #[case] expected_output: &str,
) -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = Confirm::new("Question")
        .with_default(default_value)
        .with_default_value_formatter(&|d| match d {
            true => "y".into(),
            false => "n".into(),
        })
        .prompt_with_backend(&mut backend)?;

    let rendered_frames = backend.frames();

    for (idx, frame) in rendered_frames.iter().enumerate() {
        let is_last_frame = idx == rendered_frames.len() - 1;

        if is_last_frame {
            assert!(
                frame
                    .tokens()
                    .iter()
                    .all(|t| !matches!(t, Token::DefaultValue(_))),
                "Frame {} (last) contained a help message token when it should not have",
                idx
            );
        } else {
            assert!(
                frame.has_token(&Token::DefaultValue(expected_output.into())),
                "Frame {} did not contain a help message token",
                idx
            );
        }
    }

    Ok(())
}

#[test]
fn default_help_message_does_not_exist_and_is_not_rendered() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = Confirm::new("Question")
        .with_default(true)
        .prompt_with_backend(&mut backend)?;

    let rendered_frames = backend.frames();

    for (idx, frame) in rendered_frames.iter().enumerate() {
        assert!(
            frame
                .tokens()
                .iter()
                .all(|t| !matches!(t, Token::HelpMessage(_))),
            "Frame {} contained a help message token when it should not have",
            idx
        );
    }

    Ok(())
}

#[test]
fn custom_help_message_is_rendered() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = Confirm::new("Question")
        .with_help_message("Custom help message")
        .with_default(true)
        .prompt_with_backend(&mut backend)?;

    let rendered_frames = backend.frames();

    for (idx, frame) in rendered_frames.iter().enumerate() {
        let is_last_frame = idx == rendered_frames.len() - 1;

        if is_last_frame {
            assert!(
                frame
                    .tokens()
                    .iter()
                    .all(|t| !matches!(t, Token::HelpMessage(_))),
                "Frame {} (last) contained a help message token when it should not have",
                idx
            );
        } else {
            assert!(
                frame.has_token(&Token::HelpMessage("Custom help message".into())),
                "Frame {} did not contain a help message token",
                idx
            );
        }
    }

    Ok(())
}

#[test]
fn custom_formatter_affects_final_output() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let custom_formatter = |d: bool| format!("WOW! {}", d);
    let result = Confirm::new("Question")
        .with_default(true)
        .with_formatter(&custom_formatter)
        .prompt_with_backend(&mut backend)?;

    assert!(
        result,
        "Answer selected (default) was not the expected true as default"
    );

    let final_frame = backend.frames().last().unwrap();

    assert!(
        final_frame.has_token(&Token::AnsweredPrompt(
            "Question".into(),
            "WOW! true".into()
        )),
        "Final frame did not contain the correct answer token"
    );

    Ok(())
}

#[test]
fn default_formatter_outputs_true_answer_as_yes() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let result = Confirm::new("Question")
        .with_default(true)
        .prompt_with_backend(&mut backend)?;

    assert!(
        result,
        "Answer selected (default) was not the expected true as default"
    );

    let final_frame = backend.frames().last().unwrap();

    assert!(
        final_frame.has_token(&Token::AnsweredPrompt("Question".into(), "Yes".into())),
        "Final frame did not contain the correct answer token"
    );

    Ok(())
}

#[test]
fn default_formatter_outputs_true_answer_as_no() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let result = Confirm::new("Question")
        .with_default(false)
        .prompt_with_backend(&mut backend)?;

    assert!(
        !result,
        "Answer selected (default) was not the expected false as default"
    );

    let final_frame = backend.frames().last().unwrap();

    assert!(
        final_frame.has_token(&Token::AnsweredPrompt("Question".into(), "No".into())),
        "Final frame did not contain the correct answer token"
    );

    Ok(())
}

#[test]
fn escape_raises_error() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Escape]);

    let result = Confirm::new("Question").prompt_with_backend(&mut backend);

    assert!(result.is_err(), "Result was not an error");
    assert!(
        matches!(result.unwrap_err(), InquireError::OperationCanceled),
        "Error message was not the expected one"
    );

    let final_frame = backend.frames().last().unwrap();
    assert!(
        final_frame.has_token(&Token::CanceledPrompt("Question".into())),
        "Final frame did not contain the correct canceled prompt token"
    );

    Ok(())
}

#[test]
fn ctrl_c_interrupts_prompt() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Char('c', KeyModifiers::CONTROL)]);

    let result = Confirm::new("Question").prompt_with_backend(&mut backend);

    assert!(result.is_err(), "Result was not an error");
    assert!(
        matches!(result.unwrap_err(), InquireError::OperationInterrupted),
        "Error message was not the expected one"
    );

    assert_eq!(
        1,
        backend.frames.len(),
        "Only an initial frame should have been rendered",
    );

    let final_frame = backend.frames().last().unwrap();
    assert!(
        final_frame.has_token(&Token::Prompt("Question".into())),
        "Final frame did not contain the expected prompt token"
    );

    Ok(())
}
