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

    let ans = Select::new("What's your favorite fruit?", &options)
        .with_page_size(10)
        .with_starting_cursor(1)
        .prompt()
        .expect("Failed when creating so");

    println!("Final answer was {}", ans);
}
