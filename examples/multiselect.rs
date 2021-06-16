use survey_rs::{multiselect::MultiSelect, question::Question};

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
    let mut question = MultiSelect::new("Quais frutas você vai comprar?", &options).unwrap();

    let ans = question.prompt().unwrap();

    question.cleanup(&ans).unwrap();
}
