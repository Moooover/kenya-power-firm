use anyhow::anyhow;
use async_trait::async_trait;
use chrono::naive::NaiveTime;
use chrono::NaiveDate;
use power_interuptions::location::{AffectedArea, AreaId};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

#[derive(Debug)]
pub struct Area {
    pub lines: Vec<String>,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
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

#[derive(Clone)]
pub struct Url(pub String);

pub struct ImportInput(pub HashMap<Url, Vec<Region>>);

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
    async fn notify(&self, data: ImportInput) -> anyhow::Result<()>;
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
        self.notifier.notify(data).await
    }
}
