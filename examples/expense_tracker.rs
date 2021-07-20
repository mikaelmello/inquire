use inquire::{
    error::InquireResult, parse_primitive, required, DateSelect, MultiSelect, Select, Text,
};

fn main() -> InquireResult<()> {
    let _date = DateSelect::new("Date:").prompt()?;

    let _category = Select::new("Category:", get_categories()).prompt()?;

    let _payee = Text::new("Payee:")
        .with_validator(required!("This field is required"))
        .with_suggester(&payee_suggestor)
        .with_help_message("e.g. Music Store")
        .with_page_size(5)
        .prompt()?;

    // TODO: Figure out a way to make number prompts easier to write from the user perspective.
    let _amount = Text::new("Amount:")
        .with_validator(parse_primitive!(f64, "Please type a valid number"))
        .with_formatter(&|i| format!("${}", i))
        .with_help_message("Type the amount in US dollars using a decimal point as a separator")
        .prompt()?
        .parse::<f64>()
        .unwrap();

    let _description = Text::new("Description:")
        .with_help_message("Optional notes")
        .prompt()?;

    let account = Select::new("Account:", get_accounts()).prompt()?;

    let _tags = MultiSelect::new("Tags:", get_tags()).prompt()?;

    println!("Your transaction has been successfully recorded.");
    println!("The balance of {} is now $311.09", account.value);

    Ok(())
}

/// This could be retrieved from a database, for example.
fn get_tags() -> &'static [&'static str] {
    &[
        "august-surprise",
        "birthday-gifts",
        "cat-aurora",
        "christmas-gifts-2020",
        "dog-bob",
        "dog-russ",
        "new-zealand-jan-2020",
        "roma-oct-2021",
    ]
}

/// This could be retrieved from a database, for example.
fn get_accounts() -> &'static [&'static str] {
    &[
        "401k",
        "Cash",
        "D40 Bank",
        "D40 Bank Credit Card",
        "Digital Wallet",
        "Established Bank",
        "Investments Account",
        "Meal Voucher",
        "Mortgage",
        "Zeus Bank Credit Card",
    ]
}

/// This could be retrieved from a database, for example.
fn get_categories() -> &'static [&'static str] {
    &[
        "Rent",
        "Energy",
        "Water",
        "Internet",
        "Phone",
        "Groceries",
        "Eating Out",
        "Transportation",
        "Gifts",
        "Clothes",
        "Home Appliances",
    ]
}

/// This could be faster by using smarter ways to check for matches, when dealing with larger datasets.
fn payee_suggestor(input: &str) -> Vec<String> {
    let input = input.to_lowercase();

    get_existing_payees()
        .iter()
        .filter(|p| p.to_lowercase().contains(&input))
        .take(5)
        .map(|p| String::from(*p))
        .collect()
}

/// This could be retrieved from a database, for example.
fn get_existing_payees() -> &'static [&'static str] {
    &[
        "Armstrong-Jacobs",
        "Barrows-Becker",
        "Becker PLC",
        "Bins, Fritsch and Hartmann",
        "Feil PLC",
        "Frami-Fisher",
        "Goyette Group",
        "Heathcote PLC",
        "Hilpert-Kovacek",
        "Keebler Inc",
        "Kuhn-Rippin",
        "McGlynn LLC",
        "McKenzie, Kris and Yundt",
        "Medhurst, Conroy and Will",
        "Ruecker LLC",
        "Steuber, Casper and Hermann",
        "Torphy-Boyer",
        "Volkman, Smith and Shanahan",
        "VonRueden-Rath",
        "Waelchi and Sons",
    ]
}
