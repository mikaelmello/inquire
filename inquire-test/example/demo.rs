use inquire_derive::InquireForm;

#[derive(Debug, InquireForm)]
pub struct Demo {
    #[inquire(text(
        prompt_message = "\"What's your path?\"",
        initial_value = "\"/my/initial/path\"",
        placeholder_value = "\"/my/placeholder/path\"",
    ))]
    pub path: String,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            path: "/my/default/path".to_string(),
        }
    }
}

fn main() {
    let mut ex = Demo::default();
    println!("{:?}", ex.inquire_mut().unwrap());
}
