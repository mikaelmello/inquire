use inquire::Text;

fn main() {
    let name = Text::new("What is your name?").prompt();

    match name {
        Ok(name) => println!("Hello {}", name),
        Err(_) => println!("An error happened when asking for your name, try again later."),
    }
}
