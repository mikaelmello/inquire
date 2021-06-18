use survey_rs::{
    multiselect::{MultiSelect, MultiSelectOptions},
    question::Question,
};

extern crate survey_rs;

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

    let default = vec![4, 5, 6];
    let ans = MultiSelectOptions::new("Select the fruits for your shopping list:", &options)
        .map(|mso| mso.with_help_message("This is a custom help"))
        .map(|mso| mso.with_page_size(10))
        .and_then(|mso| mso.with_default(&default))
        .and_then(|mso| mso.with_starting_cursor(1))
        .map(MultiSelect::from)
        .and_then(MultiSelect::prompt)
        .expect("Failed when creating mso");

    println!("Final answer was {}", ans);
}
