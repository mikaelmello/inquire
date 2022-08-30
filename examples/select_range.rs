use inquire::{Folder, RangeSelect};
use std::fmt::Display;

#[derive(Debug)]
struct Expenses<'a> {
    date: &'a str,
    total: usize,
}

impl<'a> Expenses<'a> {
    fn new(date: &'a str, total: usize) -> Self {
        Self { date, total }
    }
}

impl<'a> Display for Expenses<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} gold", self.date, self.total)
    }
}

fn main() {
    let options = vec![
        Expenses::new("2022-01-05", 5),
        Expenses::new("2022-01-07", 5),
        Expenses::new("2022-01-12", 5),
        Expenses::new("2022-01-18", 5),
        Expenses::new("2022-01-20", 5),
        Expenses::new("2022-01-20", 5),
        Expenses::new("2022-01-25", 5),
        Expenses::new("2022-01-25", 5),
        Expenses::new("2022-01-26", 5),
        Expenses::new("2022-01-29", 5),
        Expenses::new("2022-02-01", 5),
        Expenses::new("2022-03-02", 5),
        Expenses::new("2022-04-01", 5),
    ];

    let folder: Folder<_, String> = &|elements: &[Expenses]| {
        let costs: usize = elements.iter().map(|expenses| expenses.total).sum();
        format!("Total costs: {:?} gold", costs)
    };

    let ans = RangeSelect::new("Select effected days", options, Some(folder)).prompt_skippable();
    match ans {
        Ok(choice) => println!("your total is: {:?}", choice),
        Err(_) => println!("There was an error, please try again"),
    }
}
