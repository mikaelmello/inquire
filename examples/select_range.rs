use chrono::{Duration, TimeZone, Utc};
use inquire::{Folder, RangeSelect};
use std::fmt::Display;

#[derive(Eq, Debug)]
struct DatedDurations {
    date: chrono::DateTime<Utc>,
    duration: chrono::Duration,
}

impl DatedDurations {
    fn new(date: &str, duration: Duration) -> Self {
        Self {
            date: Utc.datetime_from_str(date, "%Y-%m-%d %H:%M:%S").unwrap(),
            duration,
        }
    }
}

impl PartialEq for DatedDurations {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date && self.duration == other.duration
    }
}

impl PartialOrd for DatedDurations {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl Ord for DatedDurations {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date)
    }
}

impl Display for DatedDurations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.date, self.duration)
    }
}

fn main() {
    let options = vec![
        DatedDurations::new("2022-01-05 12:00:00", Duration::minutes(20)),
        DatedDurations::new("2022-01-07 12:00:00", Duration::minutes(10)),
        DatedDurations::new("2022-01-12 12:00:00", Duration::minutes(50)),
        DatedDurations::new("2022-01-18 12:00:00", Duration::minutes(100)),
    ];

    let folder: Folder<_, String> = &|elements: &[DatedDurations]| {
        let full_duration = elements
            .iter()
            .map(|dated_dur| dated_dur.duration)
            .fold(Duration::zero(), |cum, element| cum + element);
        format!("Total time: {:?}", full_duration)
    };

    let ans = RangeSelect::new("Select effected days", options, Some(folder)).prompt_skippable();
    match ans {
        Ok(choice) => println!("your total is: {:?}", choice),
        Err(_) => println!("There was an error, please try again"),
    }
}
