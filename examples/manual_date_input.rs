use chrono::NaiveDate;
use inquire::{formatter::DEFAULT_DATE_FORMATTER, CustomType};

fn main() {
    let amount = CustomType::<NaiveDate>::new("When are you going to visit the office?")
        .with_placeholder("dd/mm/yyyy")
        .with_parser(&|i| NaiveDate::parse_from_str(i, "%d/%m/%Y").map_err(|_e| ()))
        .with_formatter(DEFAULT_DATE_FORMATTER)
        .with_error_message("Please type a valid date.")
        .with_help_message("The necessary arrangements will be made")
        .prompt();

    match amount {
        Ok(_) => println!("Thanks! We will be expecting you."),
        Err(_) => println!("We could not process your reservation"),
    }
}
