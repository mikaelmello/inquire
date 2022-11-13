use inquire::{
    ui::{RenderConfig, Styled},
    Confirm,
};

fn main() {
    inquire::set_global_render_config(get_render_config());

    let ans = Confirm::new("Do you live in Brazil?")
        .with_default(false)
        .with_help_message("This data is stored for good reasons")
        .prompt()
        .unwrap();

    println!("Your answer: {}", ans);

    let ans = Confirm::new("Do you want to move to another country?")
        .prompt()
        .unwrap();

    println!("Your answer: {}", ans);

    let ans = Confirm {
        message: "Are you happy?",
        default: Some(false),
        placeholder: Some("si|no"),
        help_message: Some("It's alright if you're not"),
        formatter: &|ans| match ans {
            true => "si".to_owned(),
            false => "no".to_owned(),
        },
        parser: &|ans| match ans {
            "si" => Ok(true),
            "no" => Ok(false),
            _ => Err(()),
        },
        error_message: "Reply with 'si' or 'no'".into(),
        default_value_formatter: &|def| match def {
            true => String::from("si"),
            false => String::from("no"),
        },
        render_config: get_render_config(),
    }
    .prompt()
    .unwrap();

    println!("Your answer: {}", ans);
}

fn get_render_config() -> RenderConfig<'static> {
    RenderConfig::default().with_global_prefix(Styled::new("║ "))
}
