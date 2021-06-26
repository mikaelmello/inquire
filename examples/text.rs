use inquire::{regex, Text};

fn main() {
    let name = Text::new("What is your name?")
        .with_validator(regex!("[A-Z][a-z]*", "Sorry, this name is invalid"))
        .prompt();

    match name {
        Ok(name) => println!("Hello {}", name),
        Err(_) => println!("An error happened when asking for your name, try again later."),
    }
}
