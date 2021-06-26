use inquire::{
    validator::StringValidator, Answer, AskMany, Confirm, MultiSelectOptions, Password,
    QuestionOptions, SelectOptions, Text,
};

fn main() {
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

    let input_validator: StringValidator = |ans: &str| {
        if ans.len() < 5 {
            return Err("Minimum of 5 characters");
        }

        Ok(())
    };

    let pw_validator: StringValidator = |ans| {
        if ans.len() < 8 {
            return Err("Minimum of 8 characters");
        }

        Ok(())
    };

    let workplace = Text::new("Where do you work?")
        .with_help_message("Don't worry, this will not be sold to third-party advertisers.")
        .with_validator(input_validator)
        .with_default("Unemployed")
        .prompt()
        .unwrap();

    let eats_pineapple = MultiSelectOptions::new("What are your favorite fruits?", &fruits)
        .unwrap()
        .into_question()
        .ask()
        .unwrap()
        .get_multiple_options()
        .into_iter()
        .find(|o| o.index == fruits.len() - 1)
        .is_some();

    let eats_pizza = Confirm::new("Do you eat pizza?")
        .with_default(true)
        .prompt()
        .unwrap();

    let answers =
        vec![
            SelectOptions::new("What is your favorite programming language?", &languages)
                .unwrap()
                .into_question(),
        ]
        .into_iter()
        .ask()
        .unwrap();

    let _password = Password::new("Password:")
        .with_validator(pw_validator)
        .prompt()
        .unwrap();

    let language = &answers.get(0).map(Answer::get_option).unwrap().value;

    if eats_pineapple && eats_pizza {
        println!("Thanks for your submission.\nWe have reported that a {} developer from {} puts pineapple on pizzas.", language, workplace);
    } else {
        println!("Based on our ML-powered analysis, we were able to conclude absolutely nothing.")
    }
}
