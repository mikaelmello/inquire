use chrono::NaiveDate;

use crate::DateSelect;

pub struct DateSelectConfig {
    pub vim_mode: bool,

    /// Min date allowed to be selected.
    pub min_date: Option<NaiveDate>,

    /// Max date allowed to be selected.
    pub max_date: Option<NaiveDate>,

    pub week_start: chrono::Weekday,
}

impl From<DateSelect<'_>> for DateSelectConfig {
    fn from(value: DateSelect<'_>) -> Self {
        Self {
            vim_mode: value.vim_mode,
            min_date: value.min_date,
            max_date: value.max_date,
            week_start: value.week_start,
        }
    }
}
