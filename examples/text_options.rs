use inquire::{
    error::CustomUserError,
    ui::RenderConfig,
    validator::{ExactLengthValidator, ValueRequiredValidator},
    Text,
};

fn main() {
    let answer = Text::new("What's your name?")
        .with_suggester(&suggester)
        .with_validators(&[
            Box::new(ValueRequiredValidator::new()),
            Box::new(ExactLengthValidator::new(10)),
        ])
        .prompt()
        .unwrap();

    println!("Hello {}", answer);

    let _input = Text {
        message: "How are you feeling?",
        initial_value: None,
        default: None,
        placeholder: Some("Good"),
        help_message: None,
        formatter: Text::DEFAULT_FORMATTER,
        validators: Vec::new(),
        page_size: Text::DEFAULT_PAGE_SIZE,
        suggester: None,
        completer: None,
        render_config: RenderConfig::default(),
    }
    .prompt()
    .unwrap();
}

fn suggester(val: &str) -> Result<Vec<String>, CustomUserError> {
    let suggestions = [
        "Andrew",
        "Charles",
        "Christopher",
        "Daniel",
        "David",
        "Donald",
        "Edward",
        "George",
        "James",
        "John",
        "Johnny",
        "Kevin",
        "Mark",
        "Michael",
        "Paul",
        "Richard",
        "Robert",
        "Steven",
        "Thomas",
        "William",
    ];

    let val_lower = val.to_lowercase();

    Ok(suggestions
        .iter()
        .filter(|s| s.to_lowercase().contains(&val_lower))
        .map(|s| String::from(*s))
        .collect())
}
