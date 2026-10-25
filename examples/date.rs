use chrono::{NaiveDate, Weekday};
use inquire::{formatter::DEFAULT_DATE_FORMATTER, validator::Validation, CustomType, DateSelect};

fn main() {
    date_select_default();
    custom_type_parsed_date_prompt();
    date_select_misc_options();
    date_select_with_validation();
    date_select_with_starting_date();
}

fn date_select_default() {
    println!("-------> Simple DateSelect");
    println!();

    DateSelect::new("Check-in date:").prompt().unwrap();
    println!("We will be expecting you!");
    println!();
}

fn custom_type_parsed_date_prompt() {
    println!("-------> Date parsed from text input with Custom Type prompt");
    println!();

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
    println!();
}

fn date_select_misc_options() {
    println!("-------> Date select with several possible options");
    println!();

    let date = DateSelect::new("When do you want to travel?")
        // Could also be `.with_starting_date()`
        .with_default(NaiveDate::from_ymd_opt(2021, 8, 1).unwrap())
        .with_min_date(NaiveDate::from_ymd_opt(2021, 8, 1).unwrap())
        .with_max_date(NaiveDate::from_ymd_opt(2021, 12, 31).unwrap())
        .with_week_start(Weekday::Mon)
        .with_help_message("Possible flights will be displayed according to the selected date")
        .prompt();

    match date {
        Ok(_) => println!("No flights available for this date."),
        Err(_) => println!("There was an error in the system."),
    }
    println!();
}

fn date_select_with_validation() {
    println!("-------> Date select with date validation");
    println!();

    let date = DateSelect::new("Validated input")
        .with_validator(|d: NaiveDate| {
            let now = chrono::Utc::now().naive_utc().date();

            if d.ge(&now) {
                Ok(Validation::Invalid("Date must be in the past".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt();

    match date {
        Ok(_) => println!("No flights available for this date."),
        Err(_) => println!("There was an error in the system."),
    }
    println!();
}

fn date_select_with_starting_date() {
    println!("-------> DateSelect with yesterday as initial value");
    println!();

    DateSelect::new("Check-in date:")
        // Could also be `.with_default()`
        .with_starting_date(chrono::Local::now().date_naive().pred_opt().unwrap())
        .prompt()
        .unwrap();
    println!("We will be expecting you!");
    println!();
}
