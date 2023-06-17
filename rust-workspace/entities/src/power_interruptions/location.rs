use crate::locations::LocationId;
use chrono::DateTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use shared_kernel::date_time::nairobi_date_time::NairobiTZDateTime;
use shared_kernel::date_time::time_frame::TimeFrame;
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

#[derive(Debug, Clone)]
pub struct FutureOrCurrentNairobiTZDateTime(NairobiTZDateTime);

impl AsRef<NairobiTZDateTime> for FutureOrCurrentNairobiTZDateTime {
    fn as_ref(&self) -> &NairobiTZDateTime {
        &self.0
    }
}

impl FutureOrCurrentNairobiTZDateTime {
    pub fn to_date_time(&self) -> DateTime<Tz> {
        self.0.to_date_time()
    }
}

impl From<&FutureOrCurrentNairobiTZDateTime> for NairobiTZDateTime {
    fn from(value: &FutureOrCurrentNairobiTZDateTime) -> Self {
        value.0.clone()
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

string_key!(AreaName);

#[derive(Debug, Clone)]
pub struct Area<T> {
    pub name: AreaName,
    pub time_frame: TimeFrame<T>,
    pub locations: Vec<String>,
}

#[derive(Debug)]
pub struct ImportInput(HashMap<Url, Vec<Region<FutureOrCurrentNairobiTZDateTime>>>);

impl ImportInput {
    pub fn new(data: HashMap<Url, Vec<Region<FutureOrCurrentNairobiTZDateTime>>>) -> Self {
        Self(data)
    }
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&Url, &Vec<Region<FutureOrCurrentNairobiTZDateTime>>)> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AffectedLine<T = DateTime<Tz>> {
    pub line: String,
    pub location_matched: LocationId,
    pub time_frame: TimeFrame<T>,
}

string_key!(LocationName);
