use inquire_derive::InquireForm;

#[derive(Debug, InquireForm)]
pub struct TestStruct {
    #[inquire(custom_type(prompt_message = "\"What's your name?\"",))]
    pub confirm: String,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self {
            confirm: "default_value".into(),
        }
    }
}

fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
