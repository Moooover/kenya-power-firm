mod conversion;

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::naive::NaiveTime;
use chrono::{DateTime, NaiveDate};
use chrono_tz::Tz;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use url::Url;

use crate::actor::{Actor, Permission};
use entities::power_interruptions::location::Area as DomainArea;
use entities::power_interruptions::location::NairobiTZDateTime;
use entities::power_interruptions::location::Region as DomainRegion;
use entities::power_interruptions::location::{ImportInput as DomainImportInput, TimeFrame};

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
    notifier: Arc<dyn NotifySubscribersOfAffectedAreas>,
}

impl ImportAffectedAreas {
    pub fn new(
        repo: Arc<dyn SaveBlackoutAffectedAreasRepo>,
        notifier: Arc<dyn NotifySubscribersOfAffectedAreas>,
    ) -> Self {
        Self { repo, notifier }
    }
}

#[async_trait]
impl ImportPlannedBlackoutsInteractor for ImportAffectedAreas {
    async fn import(&self, actor: &dyn Actor, data: ImportInput) -> anyhow::Result<()> {
        actor.check_for_permission(Permission::ImportAffectedAreas)?;

        let (data, errors): (Vec<_>, _) = data
            .0
            .into_iter()
            .map(|(url, regions)| {
                regions
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()
                    .map(|regions| (url.clone(), regions))
                    .with_context(|| format!("URL where data was extracted from is {}", url))
            })
            .partition(Result::is_ok);

        let data = data.into_iter().map(Result::unwrap).collect();
        let errors = errors
            .into_iter()
            .map(Result::unwrap_err)
            .collect::<Vec<_>>();

        if errors.len() > 0 {
            // TODO: Log the errors here
            println!("{errors:?}")
        }
        let data = DomainImportInput(data);
        self.repo
            .save(&data)
            .await
            .map_err(|err| anyhow!("{:?}", err))?;
        self.notifier.notify(data).await
    }
}
