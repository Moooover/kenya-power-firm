use chrono::{
    DateTime, Datelike, Days, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc,
};
use chrono_tz::Africa::Nairobi;
use chrono_tz::Tz;
use std::collections::HashMap;
use url::Url;
#[derive(Debug)]
pub struct County<T> {
    pub name: String,
    pub areas: Vec<Area<T>>,
}

#[derive(Debug)]
pub struct Region<T = FutureOrCurrentNairobiTZDateTime> {
    pub region: String,
    pub counties: Vec<County<T>>,
}

#[derive(Debug)]
pub struct NairobiTZDateTime(DateTime<Tz>);

impl NairobiTZDateTime {
    /// we only care about the date, the hour and min are set to 0 (midnight)
    fn today_at_midnight() -> Result<Self, String> {
        let today = Utc::now().naive_utc();
        Nairobi
            .with_ymd_and_hms(today.year(), today.month(), today.day(), 0, 0, 0)
            .single()
            .ok_or_else(|| format!("Failed to convert {today} to Nairobi datetime"))
            .map(NairobiTZDateTime)
    }

    fn date(&self) -> NaiveDate {
        self.0.date_naive()
    }
}

impl TryFrom<NaiveDateTime> for NairobiTZDateTime {
    type Error = String;

    fn try_from(value: NaiveDateTime) -> Result<Self, Self::Error> {
        Nairobi
            .from_local_datetime(&value)
            .single()
            .ok_or_else(|| "Failed to convert {value} to Nairobi timezone".to_string())
            .map(NairobiTZDateTime)
    }
}

#[derive(Debug)]
pub struct FutureOrCurrentNairobiTZDateTime(NairobiTZDateTime);

impl TryFrom<NairobiTZDateTime> for FutureOrCurrentNairobiTZDateTime {
    type Error = String;

    fn try_from(provided_date: NairobiTZDateTime) -> Result<Self, Self::Error> {
        let today = NairobiTZDateTime::today_at_midnight()?;
        if provided_date.date() < today.date() {
            return Err(format!(
                "The date provided already passed {:?}",
                provided_date
            ));
        }
        Ok(FutureOrCurrentNairobiTZDateTime(provided_date))
    }
}

#[derive(Debug)]
pub struct Area<T> {
    pub lines: Vec<String>,
    pub time_frame: TimeFrame<T>,
    pub locations: Vec<String>,
}

pub struct ImportInput(pub HashMap<Url, Vec<Region<FutureOrCurrentNairobiTZDateTime>>>);

#[derive(Clone, Debug)]
pub struct TimeFrame<T> {
    pub from: T,
    pub to: T,
}

#[derive(Clone)]
pub struct LocationWithDateAndTime<T = DateTime<Tz>> {
    location: String,
    area: String,
    county: String,
    time_frame: TimeFrame<T>,
}
