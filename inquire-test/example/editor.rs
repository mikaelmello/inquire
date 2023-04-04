use inquire_derive::InquireForm;

#[derive(Debug, InquireForm)]
pub struct TestStruct {
    #[inquire(editor(prompt_message = "\"What's your text?\"",))]
    pub text: String,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self {
            text: String::from("Default editor text..."),
        }
    }
}

fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
