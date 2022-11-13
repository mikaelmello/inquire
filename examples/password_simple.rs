use inquire::{
    ui::{RenderConfig, Styled},
    Password, PasswordDisplayMode,
};

fn main() {
    let render_config = RenderConfig::default().with_global_prefix(Styled::new("║ "));
    let name = Password::new("RSA Encryption Key:")
        .with_render_config(render_config)
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt();

    match name {
        Ok(_) => println!("This doesn't look like a key."),
        Err(_) => println!("An error happened when asking for your key, try again later."),
    }
}
