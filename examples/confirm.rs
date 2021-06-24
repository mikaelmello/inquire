use survey::{ConfirmOptions, Question};

extern crate survey;

fn main() {
    let ans = Question::Confirm(
        ConfirmOptions::new("Do you live in Brazil?")
            .with_default(false)
            .with_help_message("This data is stored for good reasons"),
    )
    .ask()
    .unwrap();

    println!("Your answer: {}", ans);

    let ans = Question::Confirm("Do you want to move to another country?".into())
        .ask()
        .unwrap();

    println!("Your answer: {}", ans);
}
