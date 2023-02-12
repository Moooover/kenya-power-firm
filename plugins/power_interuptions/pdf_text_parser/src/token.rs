use crate::scanner::{Date, Time};

#[derive(Debug)]
pub struct Area {
    // not so sure about this name, but it makes the most sense right now.
    // The interpreter will split this to multiple lines Vec<String>
    pub lines: String,
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
