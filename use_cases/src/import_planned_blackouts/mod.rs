use async_trait::async_trait;
use chrono::naive::NaiveTime;
use chrono::NaiveDate;
use power_interuptions::location::{AffectedArea, AreaId};
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

pub struct ImportInput(Vec<Area>);

#[async_trait]
pub trait ImportPlannedBlackoutsInteractor {
    async fn import(&self, data: ImportInput) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
pub trait SaveBlackOutsRepo {
    async fn save_blackouts(&self, data: &ImportInput) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
pub trait SubscriberNotifier {
    async fn send_notifications_to_subscribers(
        &self,
        data: Vec<AffectedArea>,
    ) -> Result<(), Box<dyn Error>>;
}

pub struct ImportBlackouts {
    repo: Arc<dyn SaveBlackOutsRepo>,
    notifier: Arc<dyn SubscriberNotifier>,
}

impl ImportPlannedBlackoutsInteractor for ImportBlackouts {
    async fn import(&self, data: ImportInput) -> Result<(), Box<dyn Error>> {
        // maybe data validation ?
        self.repo.save_blackouts(&data).await?;
        let area_ids = vec![];
        self.notifier
            .send_notifications_to_subscribers(area_ids)
            .await
    }
}
