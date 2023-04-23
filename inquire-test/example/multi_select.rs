use inquire_derive::InquireForm;

fn get_list_options() -> Vec<String> {
    vec!["rings of power".into(), "sauron".into()]
}

#[derive(Debug, InquireForm)]
#[derive(Default)]
pub struct TestStruct {
    #[inquire(multi_select(
        prompt_message = "\"What's your selection?\"",
        options = "get_list_options()"
    ))]
    pub path: Vec<String>,
}



fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
