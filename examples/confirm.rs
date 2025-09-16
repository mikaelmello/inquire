use inquire::{ui::RenderConfig, Confirm};

fn main() {
    let ans = Confirm::new("Do you live in Brazil?")
        .with_default(false)
        .with_help_message("This data is stored for good reasons")
        .prompt()
        .unwrap();

    println!("Your answer: {ans}");

    let ans = Confirm::new("Do you want to move to another country?")
        .prompt()
        .unwrap();

    println!("Your answer: {ans}");

    let ans = Confirm {
        message: "Are you happy?",
        starting_input: None,
        default: Some(false),
        placeholder: Some("sim|não"),
        help_message: Some("It's alright if you're not"),
        formatter: &|ans| match ans {
            true => "sim".to_owned(),
            false => "não".to_owned(),
        },
        parser: &|ans| match ans {
            "sim" => Ok(true),
            "não" => Ok(false),
            _ => Err(()),
        },
        error_message: "Reply with 'sim' or 'não'".into(),
        default_value_formatter: &|def| match def {
            true => String::from("sim"),
            false => String::from("não"),
        },
        render_config: RenderConfig::default(),
    }
    .prompt()
    .unwrap();

    println!("Your answer: {ans}");
}
