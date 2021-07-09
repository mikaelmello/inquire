use chrono::NaiveDate;
use inquire::DateSelect;

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
                Err("Date must be in the past".into())
            } else {
                Ok(())
            }
        })
        .prompt()
        .unwrap();
    println!("{}", date);
}
