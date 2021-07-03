use inquire::DateSelect;

fn main() {
    let date = DateSelect::new("Date:").prompt().unwrap();

    println!("{}", date);
}
