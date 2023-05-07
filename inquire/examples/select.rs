use inquire::Select;

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

    let ans = Select::new("What's your favorite fruit?", options).prompt();

    match ans {
        Ok(choice) => println!("{choice}! That's mine too!"),
        Err(_) => println!("There was an error, please try again"),
    }
}
