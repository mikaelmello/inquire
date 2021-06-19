use survey_rs::{
    ask::AskMany, confirm::ConfirmOptions, input::InputOptions, multiselect::MultiSelectOptions,
    question::Answer, select::SelectOptions,
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
            .into(),
        MultiSelectOptions::new("What are your favorite fruits?", &fruits)
            .unwrap()
            .into(),
        ConfirmOptions::new("Do you eat pizza?")
            .with_default(true)
            .into(),
        SelectOptions::new("What is the primary language you use at work?", &languages)
            .unwrap()
            .into(),
    ]
    .into_iter()
    .ask()
    .unwrap();

    let workplace = answers.get(0).and_then(Answer::get_content).unwrap();
    let eats_pineapple = answers
        .get(1)
        .and_then(Answer::get_multiple_options)
        .unwrap()
        .into_iter()
        .find(|o| o.index == fruits.len() - 1)
        .is_some();
    let eats_pizza = answers.get(2).and_then(Answer::get_confirm).unwrap();
    let language = &answers.get(3).and_then(Answer::get_option).unwrap().value;

    if eats_pineapple && eats_pizza {
        println!("After our ML-powered analysis, we conclude that {} developers from {} are much more likely to put pineapple on pizzas", language, workplace);
    } else {
        println!("After our ML-powered analysis, we were able to conclude absolutely nothing")
    }
}
