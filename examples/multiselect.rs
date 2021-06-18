use survey_rs::{
    multiselect::{MultiSelect, MultiSelectOptions},
    question::Question,
};

extern crate survey_rs;

fn main() {
    let options = vec![
        "Banana",
        "Maçã",
        "Morango",
        "Uva",
        "Limão",
        "Mexerica",
        "Melancia",
        "Laranja",
        "Pêra",
        "Jabuticaba",
        "Jaca",
    ];

    let default = vec![0, 1];
    let ans = MultiSelectOptions::new("Quais frutas você vai comprar?", &options)
        .map(|mso| mso.with_help_message("This is a custom help"))
        .map(|mso| mso.with_page_size(10))
        .and_then(|mso| mso.with_default(&default))
        .and_then(|mso| mso.with_starting_cursor(1))
        .map(MultiSelect::from)
        .and_then(MultiSelect::prompt)
        .expect("Failed when creating mso");

    println!("Final answer was {}", ans);
}
