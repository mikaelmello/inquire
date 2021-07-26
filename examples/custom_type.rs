use inquire::CustomType;

fn main() {
    let amount = CustomType::<f64>::new("How much do you want to donate?")
        .with_formatter(&|i| format!("${:.2}", i))
        .with_error_message("Please type a valid number")
        .with_help_message("Type the amount in US dollars using a decimal point as a separator")
        .prompt();

    match amount {
        Ok(_) => println!("Thanks a lot for donating that much money!"),
        Err(_) => println!("We could not process your donation"),
    }
}
