mod conversion;

use async_trait::async_trait;

use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

use crate::actor::{Actor, Permission};
use entities::power_interruptions::location::ImportInput as DomainImportInput;
use entities::power_interruptions::location::NairobiTZDateTime;

#[derive(Debug, Clone)]
pub struct Area {
    pub name: String,
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
    pub locations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct County {
    pub name: String,
    pub areas: Vec<Area>,
}
#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    pub counties: Vec<County>,
}

#[derive(Debug, Clone)]
pub struct ImportInput(pub HashMap<Url, Vec<Region>>);

#[async_trait]
pub trait ImportPlannedBlackoutsInteractor: Send + Sync {
    async fn import(&self, actor: &dyn Actor, data: ImportInput) -> anyhow::Result<()>;
}

#[async_trait]
pub trait SaveBlackoutAffectedAreasRepo: Send + Sync {
    async fn save(&self, data: &DomainImportInput) -> anyhow::Result<()>;
}

#[async_trait]
pub trait NotifySubscribersOfAffectedAreas: Send + Sync {
    async fn notify(&self, data: DomainImportInput) -> anyhow::Result<()>;
}

pub struct ImportAffectedAreas {
    repo: Arc<dyn SaveBlackoutAffectedAreasRepo>,
}

impl ImportAffectedAreas {
    pub fn new(repo: Arc<dyn SaveBlackoutAffectedAreasRepo>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ImportPlannedBlackoutsInteractor for ImportAffectedAreas {
    async fn import(&self, actor: &dyn Actor, data: ImportInput) -> anyhow::Result<()> {
        actor.check_for_permission(Permission::ImportAffectedRegions)?;

        let data = data
            .0
            .into_iter()
            .map(|(url, regions)| {
                let regions = regions.into_iter().map(Into::into).collect();
                (url, regions)
            })
            .collect();

        let data = DomainImportInput::new(data);
        self.repo.save(&data).await
    }
}
