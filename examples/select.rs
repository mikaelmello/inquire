use inquire::{
    ui::{RenderConfig, Styled},
    Select,
};

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

    let render_config = RenderConfig::default().with_global_prefix(Styled::new("║ "));
    let ans = Select::new("What's your favorite fruit?", options)
        .with_render_config(render_config)
        .prompt();

    match ans {
        Ok(choice) => println!("{}! That's mine too!", choice),
        Err(_) => println!("There was an error, please try again"),
    }
}
