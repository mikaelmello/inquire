use survey_rs::{question::Question, select::Select};

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
    let mut question = Select::new("Qual sua fruta preferida?", &options).unwrap();

    let ans = question.prompt().unwrap();

    question.cleanup(&ans).unwrap();
}
