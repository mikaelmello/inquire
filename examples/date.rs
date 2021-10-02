use chrono::NaiveDate;
use inquire::{validator::Validation, DateSelect};

fn main() {
    let date = DateSelect::new("Simple input").prompt().unwrap();
    println!("{}", date);

    let date = DateSelect::new("Date range input")
        .with_min_date(NaiveDate::from_ymd(2021, 7, 7))
        .with_max_date(NaiveDate::from_ymd(2021, 7, 12))
        .prompt()
        .unwrap();
    println!("{}", date);

    let date = DateSelect::new("With week start input")
        .with_week_start(chrono::Weekday::Mon)
        .prompt()
        .unwrap();
    println!("{}", date);

    let date = DateSelect::new("Validated input")
        .with_validator(&|d| {
            let now = chrono::Utc::now().naive_utc().date();

            if d.ge(&now) {
                Ok(Validation::Invalid("Date must be in the past".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
        .unwrap();
    println!("{}", date);
}
