use inquire::{
    ui::{RenderConfig, Styled},
    Confirm,
};

fn main() {
    let render_config = RenderConfig::default().with_global_prefix(Styled::new("║ "));
    let ans = Confirm::new("Do you live in Brazil?")
        .with_render_config(render_config)
        .with_default(false)
        .with_help_message("This data is stored for good reasons")
        .prompt();

    match ans {
        Ok(true) => println!("That's awesome!"),
        Ok(false) => println!("That's too bad, I've heard great things about it."),
        Err(_) => println!("Error with questionnaire, try again later"),
    }
}
