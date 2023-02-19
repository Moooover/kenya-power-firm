use uuid::Uuid;

pub struct Location {
    name: String,
    nearby_locations: Vec<String>,
}

pub struct LocationWithId {
    id: Uuid,
    location: Location,
}
