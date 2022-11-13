use inquire::{
    ui::{RenderConfig, Styled},
    validator::Validation,
    CustomType,
};

fn main() {
    let render_config = RenderConfig::default().with_global_prefix(Styled::new("║ "));
    let amount = CustomType::<f64>::new("How much do you want to donate?")
        .with_render_config(render_config)
        .with_formatter(&|i| format!("${:.2}", i))
        .with_error_message("Please type a valid number")
        .with_help_message("Type the amount in US dollars using a decimal point as a separator")
        .with_validator(|val: &f64| {
            if *val <= 0.0f64 {
                Ok(Validation::Invalid(
                    "You must donate a positive amount of dollars".into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt();

    match amount {
        Ok(_) => println!("Thanks a lot for donating that much money!"),
        Err(_) => println!("We could not process your donation"),
    }
}
