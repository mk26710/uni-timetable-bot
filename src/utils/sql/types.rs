use chrono::{DateTime, Datelike, FixedOffset, NaiveTime, Weekday};
use sqlx::FromRow;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "day_type", rename_all = "lowercase")]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<Weekday> for DayOfWeek {
    fn from(val: Weekday) -> Self {
        match val {
            Weekday::Mon => DayOfWeek::Monday,
            Weekday::Tue => DayOfWeek::Tuesday,
            Weekday::Wed => DayOfWeek::Wednesday,
            Weekday::Thu => DayOfWeek::Thursday,
            Weekday::Fri => DayOfWeek::Friday,
            Weekday::Sat => DayOfWeek::Saturday,
            Weekday::Sun => DayOfWeek::Sunday,
        }
    }
}

impl From<DayOfWeek> for Weekday {
    fn from(val: DayOfWeek) -> Self {
        match val {
            DayOfWeek::Monday => Weekday::Mon,
            DayOfWeek::Tuesday => Weekday::Tue,
            DayOfWeek::Wednesday => Weekday::Wed,
            DayOfWeek::Thursday => Weekday::Thu,
            DayOfWeek::Friday => Weekday::Fri,
            DayOfWeek::Saturday => Weekday::Sat,
            DayOfWeek::Sunday => Weekday::Sun,
        }
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "week_type", rename_all = "lowercase")]
pub enum WeekType {
    Odd,
    Even,
}

impl From<DateTime<FixedOffset>> for WeekType {
    fn from(val: DateTime<FixedOffset>) -> Self {
        let num = val.iso_week().week();

        match num % 2 {
            0 => WeekType::Even,
            _ => WeekType::Odd,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct MajorEntry {
    pub id: String,
    pub title: String,
    pub enrollment_year: i16,
}

#[derive(Debug, FromRow)]
pub struct TimeTableEntry {
    pub id: i64,
    pub major_id: Option<String>,
    pub week: WeekType,
    pub day_of_week: DayOfWeek,
    pub starts_at: NaiveTime,
    pub ends_at: NaiveTime,
    pub subject_name: String,
    pub subject_type: String,
    pub auditorium: String,
    pub professor: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct UserEntry {
    pub id: i64,
    pub major_id: String,
}

#[derive(Debug, FromRow)]
pub struct Exists {
    pub exists: bool,
}
