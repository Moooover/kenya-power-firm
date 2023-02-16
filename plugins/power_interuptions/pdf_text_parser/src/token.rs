use crate::scanner::{Date, Time};
use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct Area {
    pub lines: Vec<String>,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub pins: Vec<String>,
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
