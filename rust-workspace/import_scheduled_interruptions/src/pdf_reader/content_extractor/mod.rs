use crate::pdf_reader::{Area as EntityArea, County as EntityCounty, Region as EntityRegion};
use anyhow::anyhow;
use itertools::Itertools;
use shared_kernel::date_time::nairobi_date_time::FutureOrCurrentNairobiTZDateTime;
use shared_kernel::date_time::time_frame::TimeFrame;

use crate::pdf_reader::content_extractor::token::{Area, County, Region};

mod parser;
mod scanner;
mod token;

pub fn extract(text: String) -> anyhow::Result<Vec<EntityRegion>> {
    let tokens = scanner::scan(&text);
    let mut parser = parser::Parser::new(tokens);
    let result = parser.parse().map_err(|err| anyhow!("{err:?}"))?;
    Ok(result.into_iter().map_into().collect_vec())
}

impl From<Region> for EntityRegion {
    fn from(value: Region) -> Self {
        let counties = value.counties.into_iter().map(Into::into).collect();
        EntityRegion {
            region: value.name,
            counties,
        }
    }
}

impl From<County> for EntityCounty<FutureOrCurrentNairobiTZDateTime> {
    fn from(value: County) -> Self {
        let areas = value
            .areas
            .into_iter()
            .flat_map(TryFrom::try_from)
            .collect();
        EntityCounty {
            name: value.name,
            areas,
        }
    }
}

impl TryFrom<Area> for EntityArea<FutureOrCurrentNairobiTZDateTime> {
    type Error = String;

    fn try_from(value: Area) -> Result<Self, Self::Error> {
        let from = FutureOrCurrentNairobiTZDateTime::try_from(value.from)?;
        let to = FutureOrCurrentNairobiTZDateTime::try_from(value.to)?;
        Ok(EntityArea {
            name: value.name.into(),
            time_frame: TimeFrame { from, to },
            locations: value.locations,
        })
    }
}
