use inquire::{validator::MultiOptionValidator, MultiSelect};

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

    let validator: MultiOptionValidator = &|a| {
        let x = a.iter().any(|o| o.value == "Pineapple");

        match x {
            true => Ok(()),
            false => Err("Remember to buy pineapples".into()),
        }
    };

    let default = vec![4, 5, 6];
    let _ans = MultiSelect::new("Select the fruits for your shopping list:", &options)
        .with_help_message("This is a custom help")
        .with_page_size(10)
        .with_validator(validator)
        .with_default(&default)
        .with_starting_cursor(1)
        .prompt()
        .expect("Failed when creating mso");
}
