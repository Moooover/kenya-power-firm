use uuid::Uuid;

pub struct Location {
    name: String,
    nearby_locations: Vec<String>,
}

pub struct LocationId(Uuid);

pub struct LocationWithId {
    pub id: LocationId,
    pub location: Location,
}
