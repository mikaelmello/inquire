use inquire::{formatter::MultiOptionFormatter, validator::MultiOptionValidator, MultiSelect};

fn main() {
    let options = vec![
        "Banana",
        "Apple",
        "Strawberry",
        "Grapes",
        "Lemon",
        "Tangerine",
        "Watermelon",
        "Orange",
        "Pear",
        "Avocado",
        "Pineapple",
    ];

    let validator: MultiOptionValidator<&str> = &|a| {
        if a.len() < 2 {
            return Err("This list is too small!".into());
        }

        let x = a.iter().any(|o| *o.value == "Pineapple");

        match x {
            true => Ok(()),
            false => Err("Remember to buy pineapples".into()),
        }
    };

    let formatter: MultiOptionFormatter<&str> = &|a| format!("{} different fruits", a.len());

    let ans = MultiSelect::new("Select the fruits for your shopping list:", options)
        .with_validator(validator)
        .with_formatter(formatter)
        .prompt();

    match ans {
        Ok(_) => println!("I'll get right on it"),
        Err(_) => println!("The shopping list could not be processed"),
    }
}
