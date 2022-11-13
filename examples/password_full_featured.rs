use inquire::{
    min_length,
    ui::{RenderConfig, Styled},
    Password, PasswordDisplayMode,
};

fn main() {
    let render_config = RenderConfig::default().with_global_prefix(Styled::new("║ "));
    let name = Password::new("RSA Encryption Key:")
        .with_render_config(render_config)
        .with_display_toggle_enabled()
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_validator(min_length!(10))
        .with_formatter(&|_| String::from("Input received"))
        .with_help_message("It is recommended to generate a new one only for this purpose")
        .with_custom_confirmation_error_message("The keys don't match.")
        .prompt();

    match name {
        Ok(_) => println!("This doesn't look like a key."),
        Err(_) => println!("An error happened when asking for your key, try again later."),
    }
}
