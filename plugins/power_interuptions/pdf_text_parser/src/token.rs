use crate::scanner::{Date, Time};

pub struct Area {
    // not so sure about this name, but it makes the most sense right now.
    pub lines: Vec<String>,
    pub date: Date,
    pub time: Time,
    pub pins: Vec<String>,
}

pub struct County {
    pub name: String,
    pub areas: Vec<Area>,
}

pub struct Region {
    pub name: String,
    pub counties: Vec<County>,
}
