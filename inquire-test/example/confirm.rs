use inquire::Confirm;
use inquire_derive::InquireForm;

#[derive(Debug, Default, InquireForm)]
pub struct TestStruct {
    #[inquire(confirm(
        prompt_message = "\"Do you want to confirm?\"",
        default_value = "true",
        placeholder_value = "\"Y\"",
        help_message = "\"my custom helper\"",
        formatter = "Confirm::DEFAULT_FORMATTER",
        parser = "Confirm::DEFAULT_PARSER",
        default_value_formatter = "&|ans| match ans {
            true => String::from(\"Y/n\"),
            false => String::from(\"y/N\"),
        }",
        error_message = "\"my custom error message\""
    ))]
    pub confirm: bool,
}

fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
