use inquire::{validator::Validation, CustomType};

fn main() {
    let amount = CustomType::<f64>::new("How much do you want to donate?")
        .with_starting_input("10.00")
        .with_formatter(&|i| format!("${i:.2}"))
        .with_error_message("Please type a valid number")
        .with_help_message("Type the amount in US dollars using a decimal point as a separator")
        .with_validator(|val: &f64| {
            if *val <= 0.0f64 {
                Ok(Validation::Invalid(
                    "You must donate a positive amount of dollars".into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt();

    match amount {
        Ok(_) => println!("Thanks a lot for donating that much money!"),
        Err(_) => println!("We could not process your donation"),
    }
}
