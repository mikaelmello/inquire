use survey::{AskMany, InputOptions, QuestionOptions};

extern crate survey;

fn main() {
    let answers = vec![
        InputOptions::new("What's your name?")
            .with_suggestor(suggestor)
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

fn suggestor(val: &str) -> Vec<String> {
    let suggestions = vec!["Johnny", "John", "Paul", "Mark"];

    suggestions
        .into_iter()
        .map(|v| v.to_string())
        .filter(|s| s.contains(val))
        .collect()
}
