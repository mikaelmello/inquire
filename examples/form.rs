use simple_error::bail;
use survey_rs::{
    Answer, AskMany, ConfirmOptions, InputOptions, MultiSelectOptions, PasswordOptions,
    QuestionOptions, SelectOptions,
};

extern crate survey_rs;

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

    let answers = vec![
        InputOptions::new("Where do you work?")
            .with_help_message("Don't worry, this will not be sold to third-party advertisers.")
            .with_validator(|ans| match ans {
                Answer::Content(val) if val.len() < 5 => bail!("Minimum of 5 characters"),
                Answer::Content(_) => Ok(()),
                _ => panic!("Should not happen"),
            })
            .into_question(),
        MultiSelectOptions::new("What are your favorite fruits?", &fruits)
            .unwrap()
            .into_question(),
        ConfirmOptions::new("Do you eat pizza?")
            .with_default(true)
            .into_question(),
        SelectOptions::new("What is the primary language you use at work?", &languages)
            .unwrap()
            .into_question(),
        PasswordOptions::new("Password:").into_question(),
    ]
    .into_iter()
    .ask()
    .unwrap();

    let workplace = answers.get(0).map(Answer::get_content).unwrap();
    let eats_pineapple = answers
        .get(1)
        .map(Answer::get_multiple_options)
        .unwrap()
        .into_iter()
        .find(|o| o.index == fruits.len() - 1)
        .is_some();
    let eats_pizza = answers.get(2).map(Answer::get_confirm).unwrap();
    let language = &answers.get(3).map(Answer::get_option).unwrap().value;

    if eats_pineapple && eats_pizza {
        println!("After our ML-powered analysis, we conclude that {} developers from {} are much more likely to put pineapple on pizzas", language, workplace);
    } else {
        println!("After our ML-powered analysis, we were able to conclude absolutely nothing")
    }
}
