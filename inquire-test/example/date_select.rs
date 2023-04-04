use chrono::NaiveDate;
use inquire_derive::InquireForm;

#[derive(Debug, Default, InquireForm)]
pub struct TestStruct {
    #[inquire(date_select(prompt_message = "\"What's your birthday\""))]
    pub date: NaiveDate,
}

fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
