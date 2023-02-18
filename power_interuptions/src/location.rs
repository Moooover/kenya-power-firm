use chrono::{Datelike, Days, NaiveDate, NaiveTime, Utc};

pub struct County<T> {
    name: String,
    areas: Vec<Area<T>>,
}

pub struct Region<T> {
    region: String,
    counties: Vec<County<T>>,
}

pub struct FutureDate(NaiveDate);

pub struct Area<T> {
    lines: Vec<String>,
    date: T,
    time_frame: TimeFrame,
    locations: Vec<String>,
}

impl TryFrom<Area<NaiveDate>> for Area<FutureDate> {
    type Error = String;

    fn try_from(value: Area<NaiveDate>) -> Result<Self, Self::Error> {
        let provided_date = value.date;
        let today = Utc::now().date_naive();
        if provided_date < today {
            return Err(format!(
                "The date provided already passed {}",
                provided_date
            ));
        }
        Ok(Area {
            lines: value.lines,
            date: FutureDate(provided_date),
            time_frame: TimeFrame {
                from: value.time_frame.from,
                to: value.time_frame.to,
            },
            locations: value.locations,
        })
    }
}

#[derive(Clone)]
pub struct TimeFrame {
    from: NaiveTime,
    to: NaiveTime,
}

#[derive(Clone)]
pub struct LocationWithDateAndTime {
    location: String,
    area: String,
    county: String,
    date: NaiveDate,
    time_frame: TimeFrame,
}
