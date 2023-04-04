use inquire_derive::InquireForm;
use std::net::Ipv4Addr;

#[derive(Debug, InquireForm)]
pub struct TestStruct {
    #[inquire(text(prompt_message = "\"What's your path?\"",))]
    pub path: String,
    #[inquire(skip)]
    pub unsupp: Ipv4Addr,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self {
            path: "default_value".into(),
            unsupp: "192.168.2.1".parse::<Ipv4Addr>().unwrap(),
        }
    }
}

fn main() {
    let mut ex = TestStruct::default();
    ex.inquire_mut().unwrap();
    println!("{:?}", ex);
}
