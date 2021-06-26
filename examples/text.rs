use inquire::{text::PromptMany, Text};

fn main() {
    let answers = vec![
        Text::new("What's your name?").with_suggestor(suggestor),
        Text::new("What's your location?")
            .with_help_message("This data is stored for good reasons"),
    ]
    .into_iter()
    .prompt()
    .unwrap();

    println!("Hello {} from {}", answers[0], answers[1],);
}

fn suggestor(val: &str) -> Vec<String> {
    let suggestions = vec!["Johnny", "John", "Paul", "Mark"];

    suggestions
        .into_iter()
        .map(|v| v.to_string())
        .filter(|s| s.contains(val))
        .collect()
}
