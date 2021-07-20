use inquire::{required, validator::StringValidator, PromptMany, Text};

fn main() {
    let validators: &[StringValidator] = &[required!()];

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
    let mut suggestions = [
        "Johnny",
        "John",
        "Paul",
        "Mark",
        "James",
        "Robert",
        "John",
        "Michael",
        "William",
        "David",
        "Richard",
        "Thomas",
        "Charles",
        "Christopher",
        "Daniel",
        "Mark",
        "Donald",
        "Steven",
        "Paul",
        "Andrew",
        "Kevin",
        "George",
        "Edward",
    ];

    suggestions.sort();

    let val_lower = val.to_lowercase();

    suggestions
        .iter()
        .filter(|s| s.to_lowercase().contains(&val_lower))
        .map(|s| String::from(*s))
        .collect()
}
