use inquire::Password;
use inquire::PasswordDisplayMode;
use inquire_derive::InquireForm;

#[derive(Debug, Default, InquireForm)]
pub struct TestStruct {
    #[inquire(password(
        prompt_message = "\"What's your password?\"",
        help_message = "\"use your custom password\"",
        formatter = "&|_| String::from(\"xoxox\")",
        validators = "Password::DEFAULT_VALIDATORS",
        display_mode = "PasswordDisplayMode::Masked",
        enable_display_toggle = "true"
    ))]
    pub path: String,
}

fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
