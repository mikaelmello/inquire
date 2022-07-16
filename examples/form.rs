use inquire::{
    error::InquireResult, min_length, Confirm, DateSelect, MultiSelect, Password, Select, Text,
};

fn main() -> InquireResult<()> {
    let fruits = vec![
        "Banana",
        "Apple",
        "Strawberry",
        "Grapes",
        "Lemon",
        "Tangerine",
        "Watermelon",
        "Orange",
        "Pear",
        "Avocado",
        "Pineapple",
    ];
    let pineapple_index = fruits.len() - 1;

    let languages = vec![
        "C++",
        "Rust",
        "C",
        "Python",
        "Java",
        "TypeScript",
        "JavaScript",
        "Go",
    ];

    let _workplace = Text::new("Where do you work?")
        .with_help_message("Don't worry, this will not be sold to third-party advertisers.")
        .with_validator(min_length!(5, "Minimum of 5 characters"))
        .with_default("Unemployed")
        .prompt_skippable()?;

    let _eats_pineapple = MultiSelect::new("What are your favorite fruits?", fruits)
        .raw_prompt()?
        .into_iter()
        .any(|o| o.index == pineapple_index);

    let _eats_pizza = Confirm::new("Do you eat pizza?")
        .with_default(true)
        .prompt_skippable()?;

    let _language =
        Select::new("What is your favorite programming language?", languages).prompt_skippable()?;

    let _password = Password::new("Password:")
        .with_validator(min_length!(8, "Minimum of 8 characters"))
        .prompt_skippable()?;

    let _when = DateSelect::new("When are you going to travel?").prompt_skippable()?;

    println!("Based on our ML-powered analysis, we were able to conclude absolutely nothing.");

    Ok(())
}
