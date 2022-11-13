use inquire::{
    ui::{RenderConfig, Styled},
    Text,
};

fn main() {
    let render_config = RenderConfig::default().with_global_prefix(Styled::new("║ "));
    let name = Text::new("What is your name?")
        .with_render_config(render_config)
        .prompt();

    match name {
        Ok(name) => println!("Hello {}", name),
        Err(_) => println!("An error happened when asking for your name, try again later."),
    }
}
