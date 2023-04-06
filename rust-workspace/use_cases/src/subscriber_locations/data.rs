use entities::locations::LocationId;


pub struct LocationInput<T: Clone> {
    pub id: T,
    pub nearby_locations: Vec<T>,
}

impl<T: Clone> LocationInput<T> {
    pub fn primary_id(&self) -> &T {
        &self.id
    }
    pub fn ids(&self) -> Vec<T> {
        let mut ids = self.nearby_locations.to_vec();
        ids.push(self.id.clone());
        ids
    }
}

pub struct AdjuscentLocation {
    pub id: LocationId,
    pub name: String,
    pub address: String,
}

pub struct LocationWithId {
    pub id: LocationId,
    pub name: String,
    pub address: String,
    pub adjuscent_locations: Vec<AdjuscentLocation>,
}
