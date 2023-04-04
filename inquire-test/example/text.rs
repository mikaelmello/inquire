// use inquire::length;
use inquire::CustomUserError;
use inquire_derive::InquireForm;

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

#[derive(Debug, Default, InquireForm)]
pub struct TestStruct {
    #[inquire(text(
        prompt_message = "\"What's your path?\"",
        initial_value = "\"/my/initial/path\"",
        default_value = "\"/my/default/path\"",
        placeholder_value = "\"/my/placeholder/path\"",
        help_message = "\"my helper message for path\"",
        page_size = "1",
        // validators = "vec![Box::new(length!(5))]",
        autocompleter = "&suggester"
    ))]
    pub path: String,
}

fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
