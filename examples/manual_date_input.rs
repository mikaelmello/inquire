use chrono::NaiveDate;
use inquire::{
    formatter::DEFAULT_DATE_FORMATTER,
    ui::{RenderConfig, Styled},
    CustomType,
};

fn main() {
    let render_config = RenderConfig::default().with_global_prefix(Styled::new("║ "));
    let amount = CustomType::<NaiveDate>::new("When are you going to visit the office?")
        .with_render_config(render_config)
        .with_placeholder("dd/mm/yyyy")
        .with_parser(&|i| NaiveDate::parse_from_str(i, "%d/%m/%Y").map_err(|_| ()))
        .with_formatter(DEFAULT_DATE_FORMATTER)
        .with_error_message("Please type a valid date.")
        .with_help_message("The necessary arrangements will be made")
        .prompt();

    match amount {
        Ok(_) => println!("Thanks! We will be expecting you."),
        Err(_) => println!("We could not process your reservation"),
    }
}
