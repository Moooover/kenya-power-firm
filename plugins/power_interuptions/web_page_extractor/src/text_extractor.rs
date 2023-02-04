use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use std::error::Error;

pub struct Area {
    name: String,
    pins: Vec<String>,
    region: Region,
    date: NaiveDate,
    time_frame: TimeFrame,
}

pub struct Region {
    region: String,
    county: String,
}

pub struct TimeFrame {
    from: NaiveTime,
    to: NaiveTime,
}

#[async_trait]
pub trait TextExtractor {
    async fn extract(&self, text: String) -> Result<Vec<Area>, Box<dyn Error>>;
}
