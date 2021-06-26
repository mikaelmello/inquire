use inquire::{
    max_length, min_length, regex, required, text::PromptMany, validator::StringValidator, Text,
};

fn main() {
    let validators: &[StringValidator] = &[
        required!(),
        max_length!(5),
        min_length!(2),
        regex!("[A-Z][a-z]*"),
    ];

    let answers = vec![
        Text::new("What's your name?")
            .with_suggestor(suggestor)
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
