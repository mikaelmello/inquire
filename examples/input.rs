use survey_rs::{ask::Question, input::InputOptions};

extern crate survey_rs;

fn main() {
    let ans = Question::Input(
        InputOptions::new("What's your name?")
            .with_help_message("This data is stored for good reasons"),
    )
    .ask()
    .expect("Could not ask your name");

    println!("Final answer was {}", ans);
}
