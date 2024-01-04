use std::vec;

use crate::{
    date_utils::get_current_date,
    error::InquireResult,
    ui::{
        test::{FakeBackend, Token},
        Key, KeyModifiers,
    },
    validator::{ErrorMessage, Validation},
    DateSelect, InquireError,
};
use chrono::{Datelike, NaiveDate};

fn default<'a>() -> DateSelect<'a> {
    DateSelect::new("Question?")
}

macro_rules! date_test {
    ($name:ident,$input:expr,$output:expr) => {
        date_test! {$name, $input, $output, default()}
    };

    ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
        #[test]
        fn $name() -> InquireResult<()> {
            let mut backend = FakeBackend::new($input);

            let ans = $prompt.prompt_with_backend(&mut backend)?;

            assert_eq!($output, ans);

            Ok(())
        }
    };
}

date_test!(today_date, vec![Key::Enter], get_current_date());

date_test!(
    custom_default_date,
    vec![Key::Enter],
    NaiveDate::from_ymd_opt(2021, 1, 9).unwrap(),
    DateSelect::new("Date").with_default(NaiveDate::from_ymd_opt(2021, 1, 9).unwrap())
);

#[test]
/// Tests that a closure that actually closes on a variable can be used
/// as a DateSelect validator.
fn closure_validator() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter, Key::Left(KeyModifiers::NONE), Key::Enter]);

    let today_date = get_current_date();

    let validator = move |d| {
        if today_date > d {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Date must be in the past".into()))
        }
    };

    let ans = DateSelect::new("Question")
        .with_validator(validator)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(today_date.pred_opt().unwrap(), ans);

    let rendered_frames = backend.frames();
    assert!(
        rendered_frames[1].has_token(&Token::ErrorMessage(ErrorMessage::Custom(
            "Date must be in the past".into()
        )))
    );
    assert!(!rendered_frames
        .last()
        .unwrap()
        .has_token(&Token::ErrorMessage(ErrorMessage::Custom(
            "Date must be in the past".into()
        ))));

    Ok(())
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn daily_navigation_checks() -> InquireResult<()> {
    let input = vec![
        Key::Left(KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = FakeBackend::new(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(NaiveDate::from_ymd_opt(2023, 1, 20).unwrap(), ans);

    Ok(())
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn weekly_navigation_checks() -> InquireResult<()> {
    let input = vec![
        Key::Up(KeyModifiers::NONE),
        Key::Up(KeyModifiers::NONE),
        Key::Up(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = FakeBackend::new(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(NaiveDate::from_ymd_opt(2023, 2, 19).unwrap(), ans);

    Ok(())
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn monthly_navigation_checks() -> InquireResult<()> {
    let input = vec![
        Key::Char('[', KeyModifiers::NONE),
        Key::Char(']', KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Char(']', KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = FakeBackend::new(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(NaiveDate::from_ymd_opt(2022, 11, 15).unwrap(), ans);

    Ok(())
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn yearly_navigation_checks() -> InquireResult<()> {
    let input = vec![
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = FakeBackend::new(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), ans);

    Ok(())
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn naive_navigation_combination() -> InquireResult<()> {
    let input = vec![
        // start: 2023-01-15
        Key::Up(KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Up(KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Char(']', KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Left(KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Char('[', KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char(']', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Down(KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Right(KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Up(KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = FakeBackend::new(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(), ans);

    Ok(())
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn emacs_naive_navigation_combination() -> InquireResult<()> {
    let input = vec![
        // start: 2023-01-15
        Key::Char('p', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::ALT),
        Key::Char('p', KeyModifiers::CONTROL),
        Key::Char('b', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::ALT),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('b', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::ALT),
        Key::Char('b', KeyModifiers::CONTROL),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::ALT),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('v', KeyModifiers::CONTROL),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('n', KeyModifiers::CONTROL),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::CONTROL),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('p', KeyModifiers::CONTROL),
        Key::Enter,
    ];
    let mut backend = FakeBackend::new(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(), ans);

    Ok(())
}

#[test]
/// Tests the behaviour of several keybindings in an admittedly naive way.
fn vim_naive_navigation_combination() -> InquireResult<()> {
    let input = vec![
        // start: 2023-01-15
        Key::Char('k', KeyModifiers::NONE),
        Key::Char('b', KeyModifiers::ALT),
        Key::Char('k', KeyModifiers::NONE),
        Key::Char('h', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::ALT),
        Key::Char('b', KeyModifiers::ALT),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('h', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('b', KeyModifiers::ALT),
        Key::Char('h', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('b', KeyModifiers::ALT),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('f', KeyModifiers::ALT),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('j', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('{', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('l', KeyModifiers::NONE),
        Key::Char('}', KeyModifiers::NONE),
        Key::Char('k', KeyModifiers::NONE),
        Key::Enter,
    ];
    let mut backend = FakeBackend::new(input);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

    let ans = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(), ans);

    Ok(())
}

#[test]
fn default_help_message_exists_and_is_rendered() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = DateSelect::new("Question").prompt_with_backend(&mut backend)?;

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
                frame.has_token(&Token::HelpMessage(
                    DateSelect::DEFAULT_HELP_MESSAGE.unwrap().into()
                )),
                "Frame {} did not contain a help message token",
                idx
            );
        }
    }

    Ok(())
}

#[test]
fn custom_help_message_is_rendered() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = DateSelect::new("Question")
        .with_help_message("Custom help message")
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
fn removing_help_message_results_in_no_help_message_rendered() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = DateSelect::new("Question")
        .without_help_message()
        .prompt_with_backend(&mut backend)?;

    let rendered_frames = backend.frames();

    for (idx, frame) in rendered_frames.iter().enumerate() {
        assert!(
            frame
                .tokens()
                .iter()
                .all(|t| !matches!(t, Token::HelpMessage(_))),
            "Frame {} contained a help message token",
            idx
        );
    }

    Ok(())
}

#[test]
fn backend_receives_correct_default_week_start() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = DateSelect::new("Question").prompt_with_backend(&mut backend)?;

    let rendered_frames = backend.frames();

    assert_eq!(
        2,
        rendered_frames.len(),
        "Only an initial and final frame should have been rendered",
    );
    assert!(
        rendered_frames[0].tokens().iter().any(|t| matches!(
            t,
            Token::Calendar {
                week_start: DateSelect::DEFAULT_WEEK_START,
                ..
            }
        )),
        "Rendered frame did not contain a calendar token with the correct default week start",
    );

    Ok(())
}

#[test]
fn backend_receives_correct_custom_week_start() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let _ = DateSelect::new("Question")
        .with_week_start(chrono::Weekday::Wed)
        .prompt_with_backend(&mut backend)?;

    let rendered_frames = backend.frames();

    assert_eq!(
        2,
        rendered_frames.len(),
        "Only an initial and final frame should have been rendered",
    );
    assert!(
        rendered_frames[0].tokens().iter().any(|t| matches!(
            t,
            Token::Calendar {
                week_start: chrono::Weekday::Wed,
                ..
            }
        )),
        "Rendered frame did not contain a calendar token with the correct custom week start",
    );

    Ok(())
}

#[test]
fn set_min_date_is_respected() -> InquireResult<()> {
    let mut moves = vec![Key::Left(KeyModifiers::NONE); 200];
    moves.push(Key::Enter);
    let mut backend = FakeBackend::new(moves);

    let custom_min_date = NaiveDate::from_ymd_opt(2022, 12, 25).unwrap();
    let answer = DateSelect::new("Question")
        .with_starting_date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
        .with_min_date(custom_min_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(
        custom_min_date, answer,
        "Answer was not the expected custom min date"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        202,
        rendered_frames.len(),
        "Only an initial and final frame should have been rendered",
    );
    for (idx, frame) in rendered_frames[0..201].iter().enumerate() {
        assert!(frame.tokens().iter().any(
            |t| matches!(t, Token::Calendar { min_date, .. } if *min_date == Some(custom_min_date))
        ),
        "Frame {} did not contain a calendar token with the correct min date", idx);
    }

    Ok(())
}

#[test]
fn set_max_date_is_respected() -> InquireResult<()> {
    let mut moves = vec![Key::Right(KeyModifiers::NONE); 200];
    moves.push(Key::Enter);
    let mut backend = FakeBackend::new(moves);

    let custom_max_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let answer = DateSelect::new("Question")
        .with_starting_date(NaiveDate::from_ymd_opt(2023, 12, 25).unwrap())
        .with_max_date(custom_max_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(
        custom_max_date, answer,
        "Answer was not the expected custom max date"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        202,
        rendered_frames.len(),
        "Only an initial and final frame should have been rendered",
    );
    for (idx, frame) in rendered_frames[0..201].iter().enumerate() {
        assert!(frame.tokens().iter().any(
            |t| matches!(t, Token::Calendar { max_date, .. } if *max_date == Some(custom_max_date))
        ),
        "Frame {} did not contain a calendar token with the correct max date", idx);
    }

    Ok(())
}

#[test]
fn no_min_date_means_you_can_go_very_far() -> InquireResult<()> {
    let mut moves = vec![Key::Char('{', KeyModifiers::NONE); 2000]; // 2000 years back!
    moves.push(Key::Enter);
    let mut backend = FakeBackend::new(moves);

    let answer = DateSelect::new("Question")
        .with_starting_date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
        .prompt_with_backend(&mut backend)?;

    assert_eq!(
        NaiveDate::from_ymd_opt(23, 1, 1).unwrap(),
        answer,
        "Answer was not the expected custom min date"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        2002,
        rendered_frames.len(),
        "Only an initial and final frame should have been rendered",
    );
    for (idx, frame) in rendered_frames[0..2001].iter().enumerate() {
        assert!(
            frame
                .tokens()
                .iter()
                .any(|t| matches!(t, Token::Calendar { min_date: None, .. })),
            "Frame {} did not contain a calendar token with None as min date",
            idx
        );
    }

    Ok(())
}

#[test]
fn no_max_date_means_you_can_go_very_far() -> InquireResult<()> {
    let mut moves = vec![Key::Char('}', KeyModifiers::NONE); 2000]; // 2000 years forward!
    moves.push(Key::Enter);
    let mut backend = FakeBackend::new(moves);

    let answer = DateSelect::new("Question")
        .with_starting_date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
        .prompt_with_backend(&mut backend)?;

    assert_eq!(
        NaiveDate::from_ymd_opt(4023, 1, 1).unwrap(),
        answer,
        "Answer was not the expected custom min date"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        2002,
        rendered_frames.len(),
        "Only an initial and final frame should have been rendered",
    );
    for (idx, frame) in rendered_frames[0..2001].iter().enumerate() {
        assert!(
            frame
                .tokens()
                .iter()
                .any(|t| matches!(t, Token::Calendar { max_date: None, .. })),
            "Frame {} did not contain a calendar token with None as max date",
            idx
        );
    }

    Ok(())
}

#[test]
// this test might fail if `today` is set to A and the prompt is initialized
// right after the day turns, becoming A+1, but it's unlikely to happen
fn starting_date_is_today_by_default() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let today = chrono::Local::now().date_naive();
    let prompt = DateSelect::new("Question");
    assert_eq!(
        today, prompt.starting_date,
        "Starting date configured in prompt was not today"
    );

    let result = prompt.prompt_with_backend(&mut backend)?;
    assert_eq!(
        today, result,
        "Answer selected (starting_date by default) was not today"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        2,
        rendered_frames.len(),
        "Only an initial and final frame should have been rendered",
    );
    assert!(
        rendered_frames[0].tokens().iter().any(|t| matches!(
            t,
            Token::Calendar {
                selected_date,
                ..
            } if *selected_date == today
        )),
        "Rendered frame did not contain a calendar token with the correct selected date (today)",
    );

    Ok(())
}

#[test]
fn custom_starting_date_is_respected_and_selected_by_default() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let custom_starting_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let prompt = DateSelect::new("Question").with_starting_date(custom_starting_date);
    assert_eq!(
        custom_starting_date, prompt.starting_date,
        "Starting date configured in prompt was not the custom starting date"
    );

    let result = prompt.prompt_with_backend(&mut backend)?;
    assert_eq!(
        custom_starting_date, result,
        "Answer selected (starting_date by default) was not the custom starting date"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        2,
        rendered_frames.len(),
        "Only an initial and final frame should have been rendered",
    );
    assert!(
        rendered_frames[0].tokens().iter().any(|t| matches!(
            t,
            Token::Calendar {
                selected_date,
                ..
            } if *selected_date == custom_starting_date
        )),
        "Rendered frame did not contain a calendar token with the correct selected date (custom starting date)",
    );

    Ok(())
}

#[test]
fn custom_formatter_affects_final_output() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let custom_formatter = |d: NaiveDate| d.format("WOW! %Y hmm %m xd %d").to_string();
    let result = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .with_formatter(&custom_formatter)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(
        starting_date, result,
        "Answer selected (starting_date by default) was not the custom starting date"
    );

    let final_frame = backend.frames().last().unwrap();

    assert!(
        final_frame.has_token(&Token::AnsweredPrompt(
            "Question".into(),
            "WOW! 2023 hmm 01 xd 01".into()
        )),
        "Final frame did not contain the correct answer token"
    );

    Ok(())
}

#[test]
fn default_formatter_outputs_answer_as_extensive_locale() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Enter]);

    let starting_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let expected_output = starting_date.format("%B %-e, %Y").to_string();
    let result = DateSelect::new("Question")
        .with_starting_date(starting_date)
        .prompt_with_backend(&mut backend)?;

    assert_eq!(
        starting_date, result,
        "Answer selected (starting_date by default) was not the custom starting date"
    );

    let final_frame = backend.frames().last().unwrap();

    assert!(
        final_frame.has_token(&Token::AnsweredPrompt("Question".into(), expected_output)),
        "Final frame did not contain the correct answer token"
    );

    Ok(())
}

#[test]
fn escape_raises_error() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![Key::Escape]);

    let result = DateSelect::new("Question").prompt_with_backend(&mut backend);

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

    let result = DateSelect::new("Question").prompt_with_backend(&mut backend);

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

#[test]
fn validator_is_respected() -> InquireResult<()> {
    let mut backend =
        FakeBackend::new(vec![Key::Enter, Key::Right(KeyModifiers::NONE), Key::Enter]);

    let result = DateSelect::new("Question")
        .with_validator(|d: NaiveDate| {
            if d.day() % 2 == 0 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Day must be even".into()))
            }
        })
        .with_starting_date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
        .with_formatter(&|d| d.format("%Y-%m-%d").to_string())
        .prompt_with_backend(&mut backend)?;

    assert_eq!(
        NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
        result,
        "Answer selected should be initial (2023-01-01) + right (2023-01-02)"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        4,
        rendered_frames.len(),
        "Only 4 frames should have been rendered (initial, first submit, move right, final submit)",
    );
    assert!(
        rendered_frames[0]
            .tokens()
            .iter()
            .all(|t| !matches!(t, Token::ErrorMessage(ErrorMessage::Custom(_)))),
        "First frame should not have contained an error message rendered",
    );
    assert!(
        rendered_frames[1].has_token(&Token::ErrorMessage(ErrorMessage::Custom(
            "Day must be even".into()
        ))),
        "2nd frame did not contain the expected error message token",
    );
    assert!(
        rendered_frames[2].has_token(&Token::ErrorMessage(ErrorMessage::Custom(
            "Day must be even".into()
        ))),
        "3rd frame should still have the error message",
    );
    assert!(
        rendered_frames[3]
            .tokens()
            .iter()
            .all(|t| !matches!(t, Token::ErrorMessage(ErrorMessage::Custom(_)))),
        "Last frame should not have contained an error message rendered",
    );
    assert!(
        rendered_frames[3].has_token(&Token::AnsweredPrompt(
            "Question".into(),
            "2023-01-02".into()
        )),
        "Last frame did not contain the correct answer token",
    );

    Ok(())
}

#[test]
fn multiple_validators_are_respected() -> InquireResult<()> {
    let mut backend = FakeBackend::new(vec![
        Key::Enter, // 01-01
        Key::Right(KeyModifiers::NONE),
        Key::Enter, // 01-02
        Key::Right(KeyModifiers::NONE),
        Key::Enter, // 01-03
        Key::Right(KeyModifiers::NONE),
        Key::Enter, // 01-04
        Key::Right(KeyModifiers::NONE),
        Key::Enter, // 01-05
    ]);

    let result = DateSelect::new("Question")
        .with_validator(|d: NaiveDate| {
            if d == NaiveDate::from_ymd_opt(2023, 1, 2).unwrap() {
                Ok(Validation::Invalid("Must not be 2023-01-02".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .with_validator(|d: NaiveDate| {
            if d == NaiveDate::from_ymd_opt(2023, 1, 1).unwrap() {
                Ok(Validation::Invalid("Must not be 2023-01-01".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .with_validators(&[
            Box::new(|d: NaiveDate| {
                if d == NaiveDate::from_ymd_opt(2023, 1, 3).unwrap() {
                    Ok(Validation::Invalid("Must not be 2023-01-03".into()))
                } else {
                    Ok(Validation::Valid)
                }
            }),
            Box::new(|d: NaiveDate| {
                if d == NaiveDate::from_ymd_opt(2023, 1, 4).unwrap() {
                    Ok(Validation::Invalid("Must not be 2023-01-04".into()))
                } else {
                    Ok(Validation::Valid)
                }
            }),
        ])
        .with_starting_date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
        .with_formatter(&|d| d.format("%Y-%m-%d").to_string())
        .prompt_with_backend(&mut backend)?;

    assert_eq!(
        NaiveDate::from_ymd_opt(2023, 1, 5).unwrap(),
        result,
        "Answer selected should be initial (2023-01-01) + right (2023-01-02)"
    );

    let rendered_frames = backend.frames();

    assert_eq!(
        10,
        rendered_frames.len(),
        "Only 4 frames should have been rendered (initial, first submit, move right, final submit)",
    );
    assert!(
        rendered_frames[0]
            .tokens()
            .iter()
            .all(|t| !matches!(t, Token::ErrorMessage(ErrorMessage::Custom(_)))),
        "First frame should not have contained an error message rendered",
    );
    #[allow(clippy::needless_range_loop)]
    for frame in 1..3 {
        assert!(
            rendered_frames[frame].has_token(&Token::ErrorMessage(ErrorMessage::Custom(
                "Must not be 2023-01-01".into()
            ))),
            "Expected to find error message of first validator in frame {}",
            frame
        );
    }
    #[allow(clippy::needless_range_loop)]
    for frame in 3..5 {
        assert!(
            rendered_frames[frame].has_token(&Token::ErrorMessage(ErrorMessage::Custom(
                "Must not be 2023-01-02".into()
            ))),
            "Expected to find error message of second validator in frame {}",
            frame
        );
    }
    #[allow(clippy::needless_range_loop)]
    for frame in 5..7 {
        assert!(
            rendered_frames[frame].has_token(&Token::ErrorMessage(ErrorMessage::Custom(
                "Must not be 2023-01-03".into()
            ))),
            "Expected to find error message of second validator in frame {}",
            frame
        );
    }
    #[allow(clippy::needless_range_loop)]
    for frame in 7..9 {
        assert!(
            rendered_frames[frame].has_token(&Token::ErrorMessage(ErrorMessage::Custom(
                "Must not be 2023-01-04".into()
            ))),
            "Expected to find error message of second validator in frame {}",
            frame
        );
    }
    assert!(
        rendered_frames[9]
            .tokens()
            .iter()
            .all(|t| !matches!(t, Token::ErrorMessage(ErrorMessage::Custom(_)))),
        "Last frame should not have contained an error message rendered",
    );
    assert!(
        rendered_frames[9].has_token(&Token::AnsweredPrompt(
            "Question".into(),
            "2023-01-05".into()
        )),
        "Last frame did not contain the correct answer token",
    );

    Ok(())
}
