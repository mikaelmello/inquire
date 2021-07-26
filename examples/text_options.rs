use inquire::{required, Text};

fn main() {
    let answer = Text::new("What's your name?")
        .with_suggester(&suggester)
        .with_validators(&[required!()])
        .prompt()
        .unwrap();

    println!("Hello {}", answer);

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
