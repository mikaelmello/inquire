use core::panic;

use chrono::NaiveDate;

pub fn get_current_date() -> NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn get_start_date(month: chrono::Month, year: i32) -> NaiveDate {
    chrono::NaiveDate::from_ymd_opt(year, month.number_from_month(), 1).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_date() {
        let current_date = get_current_date();
        let expected_date = chrono::Local::now().date_naive();
        assert_eq!(current_date, expected_date);
    }

    #[test]
    fn test_get_start_date() {
        assert_eq!(
            get_start_date(chrono::Month::January, 2021),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()
        );
        assert_eq!(
            get_start_date(chrono::Month::February, 2021),
            chrono::NaiveDate::from_ymd_opt(2021, 2, 1).unwrap()
        );
        assert_eq!(
            get_start_date(chrono::Month::March, 2021),
            chrono::NaiveDate::from_ymd_opt(2021, 3, 1).unwrap()
        );
        assert_eq!(
            get_start_date(chrono::Month::December, 1883),
            chrono::NaiveDate::from_ymd_opt(1883, 12, 1).unwrap()
        );
        assert_eq!(
            get_start_date(chrono::Month::June, 3042),
            chrono::NaiveDate::from_ymd_opt(3042, 6, 1).unwrap()
        );
    }

    #[test]
    // this is basically a reimplementation but it works as a sanity check
    fn test_get_month() {
        assert_eq!(get_month(1), chrono::Month::January);
        assert_eq!(get_month(2), chrono::Month::February);
        assert_eq!(get_month(3), chrono::Month::March);
        assert_eq!(get_month(4), chrono::Month::April);
        assert_eq!(get_month(5), chrono::Month::May);
        assert_eq!(get_month(6), chrono::Month::June);
        assert_eq!(get_month(7), chrono::Month::July);
        assert_eq!(get_month(8), chrono::Month::August);
        assert_eq!(get_month(9), chrono::Month::September);
        assert_eq!(get_month(10), chrono::Month::October);
        assert_eq!(get_month(11), chrono::Month::November);
        assert_eq!(get_month(12), chrono::Month::December);
    }

    #[test]
    #[should_panic]
    fn test_get_month_0_panics() {
        get_month(0);
    }

    #[test]
    #[should_panic]
    fn test_get_month_13_panics() {
        get_month(13);
    }
}
