use chrono::{DateTime, Datelike, Days, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
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
pub struct Region<T = FutureOrCurrentNairobiDateTime> {
    pub region: String,
    pub counties: Vec<County<T>>,
}

#[derive(Debug)]
pub struct NairobiDateTime(DateTime<Tz>);

impl NairobiDateTime {
    fn now() -> Result<Self, String> {
        let today = Utc::now().naive_utc();
        Nairobi
            .from_local_datetime(&today)
            .single()
            .ok_or_else(|| format!("Failed to convert {today} to Nairobi datetime"))
            .map(NairobiDateTime)
    }

    fn date(&self) -> NaiveDate {
        self.0.date_naive()
    }
}

impl TryFrom<NaiveDateTime> for NairobiDateTime {
    type Error = String;

    fn try_from(value: NaiveDateTime) -> Result<Self, Self::Error> {
        Nairobi
            .from_local_datetime(&value)
            .single()
            .ok_or_else(|| "Failed to convert {value} to Nairobi timezone".to_string())
            .map(NairobiDateTime)
    }
}

#[derive(Debug)]
pub struct FutureOrCurrentNairobiDateTime(NairobiDateTime);

impl TryFrom<NairobiDateTime> for FutureOrCurrentNairobiDateTime {
    type Error = String;

    fn try_from(provided_date: NairobiDateTime) -> Result<Self, Self::Error> {
        let now = NairobiDateTime::now()?;
        if provided_date.date() < now.date() {
            return Err(format!(
                "The date provided already passed {:?}",
                provided_date
            ));
        }
        Ok(FutureOrCurrentNairobiDateTime(provided_date))
    }
}

#[derive(Debug)]
pub struct Area<T> {
    pub lines: Vec<String>,
    pub time_frame: TimeFrame<T>,
    pub locations: Vec<String>,
}

pub struct ImportInput(pub HashMap<Url, Vec<Region<FutureOrCurrentNairobiDateTime>>>);

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
