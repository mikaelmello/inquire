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

    // 默认是 true，直接回车提交
    let result = Confirm::from("Question")
        .with_default(true)
        .prompt_with_backend(&mut backend)?;
    assert!(result, "Answer was not the expected one");

    Ok(())
}

#[rstest]
#[case('y', true)]
#[case('Y', true)]
#[case('n', false)]
#[case('N', false)]
fn prompt_correctly_handles_direct_hotkeys(
    #[case] key_char: char,
    #[case] expected_result: bool,
) -> InquireResult<()> {
    // 🚀 核心测试：单击字符 y/n 就会立即触发极速提交，根本不需要 Enter
    let mut backend = FakeBackend::new(vec![Key::Char(key_char, KeyModifiers::NONE)]);

    let result = Confirm::from("Question")
        .with_default(!expected_result) // 故意把默认值设反，确保是单键起效了
        .prompt_with_backend(&mut backend)?;
    assert_eq!(expected_result, result, "Answer was not the expected one");

    Ok(())
}

#[rstest]
fn escape_after_successful_submit_has_no_effect() -> InquireResult<()> {
    // 按下 'y' 触发极速提交后，后续的 Escape 动作不应该有任何破坏性副作用
    let mut backend = FakeBackend::new(vec![Key::Char('y', KeyModifiers::NONE), Key::Escape]);

    let result = Confirm::from("Question")
        .with_default(false)
        .prompt_with_backend(&mut backend)?;
    assert!(result, "Answer was not the expected one");

    Ok(())
}

#[rstest]
#[case(true)]
#[case(false)]
fn prompt_with_default_can_be_readily_submitted(#[case] default_value: bool) -> InquireResult<()> {
    // 直接敲回车，应该能够安全地直接提交设置好的默认布尔值
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let result = Confirm::from("Question")
        .with_default(default_value)
        .prompt_with_backend(&mut backend)?;
    assert_eq!(default_value, result, "Answer was not the expected one");

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
                "Frame {} (last) contained a default value token when it should not have",
                idx
            );
        } else {
            assert!(
                frame.has_token(&Token::DefaultValue(expected_output.into())),
                "Frame {} did not contain a default value token",
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
                "Frame {} (last) contained a default value token when it should not have",
                idx
            );
        } else {
            assert!(
                frame.has_token(&Token::DefaultValue(expected_output.into())),
                "Frame {} did not contain a default value token",
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
        let has_unexpected_help = frame.tokens().iter().any(|t| {
            if let Token::HelpMessage(msg) = t {
                !msg.is_empty() // 只有非空才算“渲染了帮助信息”
            } else {
                false
            }
        });

        assert!(
            !has_unexpected_help,
            "Frame {} contained an unexpected active help message token",
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
fn default_formatter_outputs_false_answer_as_no() -> InquireResult<()> {
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
