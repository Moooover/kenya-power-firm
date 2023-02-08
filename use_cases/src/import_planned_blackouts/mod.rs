use anyhow::anyhow;
use async_trait::async_trait;
use chrono::naive::NaiveTime;
use chrono::NaiveDate;
use power_interuptions::location::{AffectedArea, AreaId};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
pub struct Pin(String);

pub struct Region {
    region: String,
    part: String,
}

pub struct Area {
    name: String,
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
    pins: Vec<Pin>,
    region: Region,
}

pub struct Url(pub String);

pub struct ImportInput(pub HashMap<Url, Vec<Area>>);

#[async_trait]
pub trait ImportPlannedBlackoutsInteractor {
    async fn import(&self, data: ImportInput) -> anyhow::Result<()>;
}

#[async_trait]
pub trait SaveBlackOutsRepo: Send + Sync {
    async fn save_blackouts(&self, data: &ImportInput) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
pub trait NotifySubscribersOfAffectedAreas: Send + Sync {
    async fn notify(&self, data: Vec<AffectedArea>) -> Result<(), Box<dyn Error>>;
}

pub struct ImportBlackouts {
    repo: Arc<dyn SaveBlackOutsRepo>,
    notifier: Arc<dyn NotifySubscribersOfAffectedAreas>,
}

#[async_trait]
impl ImportPlannedBlackoutsInteractor for ImportBlackouts {
    async fn import(&self, data: ImportInput) -> anyhow::Result<()> {
        // maybe data validation ?
        self.repo
            .save_blackouts(&data)
            .await
            .map_err(|err| anyhow!("{:?}", err))?;
        let area_ids = vec![];
        self.notifier
            .notify(area_ids)
            .await
            .map_err(|err| anyhow!("{:?}", err))
    }
}
