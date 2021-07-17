use inquire::{max_length, min_length, required, validator::StringValidator, PromptMany, Text};

fn main() {
    let validators: &[StringValidator] = &[required!(), max_length!(5), min_length!(2)];

    let answers = vec![
        Text::new("What's your name?")
            .with_suggester(&suggester)
            .with_validators(validators),
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
        validators: Vec::new(),
        page_size: Text::DEFAULT_PAGE_SIZE,
        suggester: None,
    }
    .prompt()
    .unwrap();
}

fn suggester(val: &str) -> Vec<String> {
    let suggestions = vec!["Johnny", "John", "Paul", "Mark"];

    suggestions
        .into_iter()
        .map(|v| v.to_string())
        .filter(|s| s.contains(val))
        .collect()
}
