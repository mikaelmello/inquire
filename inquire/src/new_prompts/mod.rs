use chrono::NaiveDate;

use self::date_prompt::DateSelect;

pub mod api;
pub mod base;
pub mod date_prompt;
pub mod text_prompt;

#[test]
fn test() {
    let pr = DateSelect::new("Testing")
        .with_min_date(NaiveDate::from_ymd_opt(2023, 12, 10).unwrap())
        .prompt()
        .unwrap();

    println!("{:?}", pr);
}
