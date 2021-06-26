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

    println!("Hello {} from {}", answers[0], answers[1]);

    let _input = Text {
        message: "How are you feeling?",
        default: None,
        help_message: None,
        formatter: Text::DEFAULT_FORMATTER,
        validator: None,
        page_size: Text::DEFAULT_PAGE_SIZE,
        suggestor: None,
    }
    .prompt()
    .unwrap();
}

fn suggestor(val: &str) -> Vec<String> {
    let suggestions = vec!["Johnny", "John", "Paul", "Mark"];

    suggestions
        .into_iter()
        .map(|v| v.to_string())
        .filter(|s| s.contains(val))
        .collect()
}
