use crate::locations::LocationId;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Africa::Nairobi;
use chrono_tz::Tz;
use shared_kernel::string_key;
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone)]
pub struct County<T> {
    pub name: String,
    pub areas: Vec<Area<T>>,
}

#[derive(Debug)]
pub struct Region<T = FutureOrCurrentNairobiTZDateTime> {
    pub region: String,
    pub counties: Vec<County<T>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct NairobiTZDateTime(DateTime<Tz>);

impl NairobiTZDateTime {
    pub fn today() -> Self {
        let today = Utc::now().naive_utc();
        NairobiTZDateTime(Nairobi.from_utc_datetime(&today))
    }

    fn date(&self) -> NaiveDate {
        self.0.date_naive()
    }

    pub fn to_date_time(&self) -> DateTime<Tz> {
        self.0
    }
}

impl From<DateTime<Utc>> for NairobiTZDateTime {
    fn from(data: DateTime<Utc>) -> NairobiTZDateTime {
        let data = data.naive_utc();
        NairobiTZDateTime(Nairobi.from_utc_datetime(&data))
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

#[derive(Debug, Clone)]
pub struct FutureOrCurrentNairobiTZDateTime(NairobiTZDateTime);

impl From<&FutureOrCurrentNairobiTZDateTime> for NairobiTZDateTime {
    fn from(value: &FutureOrCurrentNairobiTZDateTime) -> Self {
        value.0.clone()
    }
}

impl FutureOrCurrentNairobiTZDateTime {
    pub fn to_date_time(&self) -> DateTime<Tz> {
        self.0.to_date_time()
    }
}

impl TryFrom<NairobiTZDateTime> for FutureOrCurrentNairobiTZDateTime {
    type Error = String;

    fn try_from(provided_date: NairobiTZDateTime) -> Result<Self, Self::Error> {
        let today = NairobiTZDateTime::today();
        if provided_date.date() < today.date() {
            return Err(format!(
                "The date provided already passed {:?}",
                provided_date
            ));
        }
        Ok(FutureOrCurrentNairobiTZDateTime(provided_date))
    }
}

#[derive(Debug, Clone)]
pub struct Area<T> {
    pub name: String,
    pub time_frame: TimeFrame<T>,
    pub locations: Vec<String>,
}

pub struct ImportInput(pub HashMap<Url, Vec<Region<FutureOrCurrentNairobiTZDateTime>>>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TimeFrame<T> {
    pub from: T,
    pub to: T,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AffectedLine<T = DateTime<Tz>> {
    pub line: String,
    pub location_matched: LocationId,
    pub time_frame: TimeFrame<T>,
}

string_key!(LocationName);

string_key!(AreaName);
