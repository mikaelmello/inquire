use inquire_derive::InquireForm;

fn get_list_options() -> Vec<String> {
    vec!["rings of power".into(), "sauron".into()]
}

#[derive(Debug, InquireForm)]
pub struct TestStruct {
    #[inquire(select(
        prompt_message = "\"What's your selection?\"",
        options = "get_list_options()"
    ))]
    pub path: String,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self {
            path: String::from("test"),
        }
    }
}

fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
