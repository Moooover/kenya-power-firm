


use entities::power_interruptions::location::NairobiTZDateTime;

#[derive(Debug)]
pub struct Area {
    pub name: String,
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
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
