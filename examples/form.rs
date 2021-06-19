use survey_rs::{ask::AskMany, multiselect::MultiSelectOptions, select::SelectOptions};

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

    let questions = vec![
        SelectOptions::new("What's your favorite fruit?", &fruits)
            .unwrap()
            .into(),
        MultiSelectOptions::new("Which languages do you use at work?", &languages)
            .unwrap()
            .into(),
    ]
    .into_iter();

    let answers = questions.ask().unwrap();

    for (i, ans) in answers.iter().enumerate() {
        println!("Ans #{} is {}", i, ans);
    }
}
