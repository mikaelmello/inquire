use inquire::Confirm;

fn main() {
    let ans = Confirm::new("Do you live in Brazil?")
        .with_default(false)
        .with_help_message("This data is stored for good reasons")
        .prompt()
        .unwrap();

    println!("Your answer: {}", ans);

    let ans = Confirm::new("Do you want to move to another country?")
        .prompt()
        .unwrap();

    println!("Your answer: {}", ans);

    let ans = Confirm {
        message: "Are you happy?",
        default: Some(false),
        help_message: Some("It's alright if you're not"),
        formatter: Confirm::DEFAULT_FORMATTER,
        parser: |ans| match ans {
            "si" => Ok(true),
            "no" => Ok(false),
            _ => Err("Reply with 'si' or 'no'".into()),
        },
        default_value_formatter: |def| match def {
            true => "si",
            false => "no",
        },
    }
    .prompt()
    .unwrap();

    println!("Your answer: {}", ans);
}
