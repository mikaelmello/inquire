use inquire::DateSelect;

fn main() {
    let date = DateSelect::new("When do you want to travel?")
        .with_default(chrono::NaiveDate::from_ymd(2021, 8, 1))
        .with_min_date(chrono::NaiveDate::from_ymd(2021, 8, 1))
        .with_max_date(chrono::NaiveDate::from_ymd(2021, 12, 31))
        .with_week_start(chrono::Weekday::Mon)
        .with_help_message("Possible flights will be displayed according to the selected date")
        .prompt();

    match date {
        Ok(_) => println!("No flights available for this date."),
        Err(_) => println!("There was an error in the system."),
    }
}
