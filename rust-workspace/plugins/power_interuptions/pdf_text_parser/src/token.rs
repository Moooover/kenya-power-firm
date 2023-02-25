use crate::scanner::{Date, Time};
use chrono::{DateTime, NaiveDate, NaiveTime};
use chrono_tz::Tz;
use power_interuptions::location::NairobiDateTime;

#[derive(Debug)]
pub struct Area {
    pub lines: Vec<String>,
    pub from: NairobiDateTime,
    pub to: NairobiDateTime,
    pub locations: Vec<String>,
}

#[derive(Debug)]
pub struct County {
    pub name: String,
    pub areas: Vec<Area>,
}
#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub counties: Vec<County>,
}
