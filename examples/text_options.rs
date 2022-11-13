use inquire::{
    error::CustomUserError,
    length, required,
    ui::{RenderConfig, Styled},
    Text,
};

fn main() {
    let answer = Text::new("What's your name?")
        .with_render_config(get_render_config())
        .with_autocomplete(&suggester)
        .with_validator(required!())
        .with_validator(length!(10))
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
        autocompleter: None,
        render_config: get_render_config(),
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

fn get_render_config() -> RenderConfig<'static> {
    RenderConfig::default().with_global_prefix(Styled::new("║ "))
}
