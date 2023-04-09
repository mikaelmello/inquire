use core::panic;

use chrono::NaiveDate;

pub fn get_current_date() -> NaiveDate {
    chrono::Local::now().date().naive_local()
}

pub fn get_start_date(month: chrono::Month, year: i32) -> NaiveDate {
    chrono::NaiveDate::from_ymd(year, month.number_from_month(), 1)
}

pub fn get_month(month: u32) -> chrono::Month {
    match month {
        1 => chrono::Month::January,
        2 => chrono::Month::February,
        3 => chrono::Month::March,
        4 => chrono::Month::April,
        5 => chrono::Month::May,
        6 => chrono::Month::June,
        7 => chrono::Month::July,
        8 => chrono::Month::August,
        9 => chrono::Month::September,
        10 => chrono::Month::October,
        11 => chrono::Month::November,
        12 => chrono::Month::December,
        _ => panic!("Invalid month"),
    }
}
