use shared_kernel::date_time::nairobi_date_time::NairobiTZDateTime;
use shared_kernel::date_time::time_frame::TimeFrame;

#[derive(Debug)]
pub struct Area {
    pub name: String,
    pub time_frame: Vec<TimeFrame<NairobiTZDateTime>>,
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
