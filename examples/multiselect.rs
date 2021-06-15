use std::fmt::Debug;

use survey_rs::{multiselect::MultiSelect, question::Question};

extern crate survey_rs;

// examples/hello.rs
fn main() {
    let options = vec![
        "Option",
        "Opción",
        "Opção",
        "Escolha",
        "Choice",
        "Selección",
        "Seleção",
        "Selection",
        "Mikael",
        "Bruna",
        "Mãe",
        "Millene",
        "Pai",
    ];
    let mut question = MultiSelect::new("Qual você quer escolher?", &options).unwrap();

    let ans = question.prompt().unwrap();

    println!("{:?}", ans);
}
