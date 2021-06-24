use simple_error::SimpleError;
use survey::{config::Validator, Answer, MultiSelectOptions, Question};

extern crate survey;

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

    let validator: Validator = |a| match a {
        Answer::MultipleOptions(opt) => {
            let x = opt.iter().any(|o| o.value == "Pineapple");

            match x {
                true => Ok(()),
                false => Err(Box::new(SimpleError::new("Remember to buy pineapples"))),
            }
        }
        _ => panic!("Invalid answer"),
    };

    let default = vec![4, 5, 6];
    let ans = MultiSelectOptions::new("Select the fruits for your shopping list:", &options)
        .map(|mso| mso.with_help_message("This is a custom help"))
        .map(|mso| mso.with_page_size(10))
        .map(|mso| mso.with_validator(validator))
        .and_then(|mso| mso.with_default(&default))
        .and_then(|mso| mso.with_starting_cursor(1))
        .map(Question::MultiSelect)
        .and_then(Question::ask)
        .expect("Failed when creating mso");

    println!("Final answer was {}", ans);
}
