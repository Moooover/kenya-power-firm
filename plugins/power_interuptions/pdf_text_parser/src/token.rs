use crate::scanner::{Date, Time};

#[derive(Debug)]
pub struct Area {
    pub lines: Vec<String>,
    pub date: Date,
    pub time: Time,
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
