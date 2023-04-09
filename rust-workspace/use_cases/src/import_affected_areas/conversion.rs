use crate::import_affected_areas::{Area, County, Region};
use entities::power_interruptions::location::{
    Area as DomainArea, County as DomainCounty, FutureOrCurrentNairobiTZDateTime,
    Region as DomainRegion, TimeFrame,
};

impl From<Region> for DomainRegion<FutureOrCurrentNairobiTZDateTime> {
    fn from(value: Region) -> Self {
        let counties = value.counties.into_iter().map(Into::into).collect();
        Self {
            region: value.name,
            counties,
        }
    }
}

impl From<County> for DomainCounty<FutureOrCurrentNairobiTZDateTime> {
    fn from(value: County) -> Self {
        let areas = value
            .areas
            .into_iter()
            .flat_map(TryFrom::try_from)
            .collect();
        DomainCounty {
            name: value.name,
            areas,
        }
    }
}

impl TryFrom<Area> for DomainArea<FutureOrCurrentNairobiTZDateTime> {
    type Error = String;

    fn try_from(value: Area) -> Result<Self, Self::Error> {
        let from = FutureOrCurrentNairobiTZDateTime::try_from(value.from)?;
        let to = FutureOrCurrentNairobiTZDateTime::try_from(value.to)?;
        Ok(DomainArea {
            name: value.name.into(),
            time_frame: TimeFrame { from, to },
            locations: value.locations,
        })
    }
}
