use chrono::{NaiveDate, NaiveTime};

// TODO: Refactor this to UUID later

pub struct PinId(String);

pub struct Pin(PinId);

pub struct RegionId(String);
pub struct CountyId(String);

pub struct Region {
    region: RegionId,
    county: CountyId,
}

pub struct AreaId(String);

pub struct Area {
    name: AreaId,
    pins: Vec<Pin>,
    region: Region,
}

#[derive(Clone)]
pub struct TimeFrame {
    from: NaiveTime,
    to: NaiveTime,
}

pub struct AffectedArea {
    area: AreaId,
    date: NaiveDate,
    time_frame: TimeFrame,
}

#[derive(Clone)]
pub struct LocationWithDateAndTime {
    location: String,
    area: String,
    county: String,
    date: NaiveDate,
    time_frame: TimeFrame,
}
