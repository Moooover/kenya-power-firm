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
use power_interuptions::location::Area as DomainArea;
use power_interuptions::location::Region as DomainRegion;
use power_interuptions::location::{
    County as DomainCounty, FutureOrCurrentNairobiDateTime, NairobiDateTime,
};
use power_interuptions::location::{ImportInput as DomainImportInput, TimeFrame};

#[derive(Debug)]
pub struct Area {
    pub lines: Vec<String>,
    pub from: NairobiDateTime,
    pub to: NairobiDateTime,
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
pub trait SaveBlackOutsRepo: Send + Sync {
    async fn save_blackouts(&self, data: &DomainImportInput) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
pub trait NotifySubscribersOfAffectedAreas: Send + Sync {
    async fn notify(&self, data: DomainImportInput) -> anyhow::Result<()>;
}

pub struct ImportBlackouts {
    repo: Arc<dyn SaveBlackOutsRepo>,
    notifier: Arc<dyn NotifySubscribersOfAffectedAreas>,
}

#[async_trait]
impl ImportPlannedBlackoutsInteractor for ImportBlackouts {
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
            .save_blackouts(&data)
            .await
            .map_err(|err| anyhow!("{:?}", err))?;
        self.notifier.notify(data).await
    }
}

impl TryFrom<Region> for DomainRegion<FutureOrCurrentNairobiDateTime> {
    type Error = anyhow::Error;

    fn try_from(value: Region) -> Result<Self, Self::Error> {
        let counties = value
            .counties
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_, _>>()
            .with_context(|| format!("Region {}", value.name))?;
        Ok(Self {
            region: value.name,
            counties,
        })
    }
}

impl TryFrom<County> for DomainCounty<FutureOrCurrentNairobiDateTime> {
    type Error = anyhow::Error;

    fn try_from(value: County) -> Result<Self, Self::Error> {
        let areas = value
            .areas
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_, _>>()
            .with_context(|| format!("County {}", value.name))?;
        Ok(DomainCounty {
            name: value.name,
            areas,
        })
    }
}

impl TryFrom<Area> for DomainArea<FutureOrCurrentNairobiDateTime> {
    type Error = anyhow::Error;

    fn try_from(value: Area) -> Result<Self, Self::Error> {
        let from =
            FutureOrCurrentNairobiDateTime::try_from(value.from).map_err(|error| anyhow!(error))?;
        let to = FutureOrCurrentNairobiDateTime::try_from(value.to).map_err(|err| anyhow!(err))?;
        Ok(DomainArea {
            lines: value.lines,
            time_frame: TimeFrame { from, to },
            locations: value.locations,
        })
    }
}
