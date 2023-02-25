use chrono::{DateTime, Datelike, Days, NaiveDate, NaiveTime, TimeZone, Utc};
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
pub struct Region<T = FutureOrCurrentDate> {
    pub region: String,
    pub counties: Vec<County<T>>,
}

#[derive(Debug)]
pub struct FutureOrCurrentDate(DateTime<Tz>);

#[derive(Debug)]
pub struct Area<T> {
    pub lines: Vec<String>,
    pub time_frame: TimeFrame<T>,
    pub locations: Vec<String>,
}

pub struct ImportInput(pub HashMap<Url, Vec<Region<FutureOrCurrentDate>>>);

impl TryFrom<DateTime<Tz>> for FutureOrCurrentDate {
    type Error = String;

    fn try_from(provided_date: DateTime<Tz>) -> Result<Self, Self::Error> {
        use chrono_tz::Africa::Nairobi;

        let today = Utc::now().naive_utc();
        let today = Nairobi
            .from_local_datetime(&today)
            .single()
            .ok_or_else(|| format!("Failed to convert {provided_date} to Nairobi datetime"))?;
        if provided_date < today {
            return Err(format!(
                "The date provided already passed {}",
                provided_date
            ));
        }
        Ok(FutureOrCurrentDate(provided_date))
    }
}

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
