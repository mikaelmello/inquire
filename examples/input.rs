use survey_rs::{AskMany, InputOptions, QuestionOptions};

extern crate survey_rs;

fn main() {
    let answers = vec![
        InputOptions::new("What's your name?")
            .with_help_message("This data is stored for good reasons")
            .into_question(),
        InputOptions::new("What's your location?")
            .with_help_message("This data is stored for good reasons")
            .into_question(),
    ]
    .into_iter()
    .ask()
    .unwrap();

    println!(
        "Hello {} from {}",
        answers[0].get_content(),
        answers[1].get_content(),
    );
}
