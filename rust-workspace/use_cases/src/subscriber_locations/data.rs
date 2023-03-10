use uuid::Uuid;

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

#[derive(Copy, Clone, Debug)]
pub struct LocationId(Uuid);

impl LocationId {
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for LocationId {
    fn from(value: Uuid) -> Self {
        LocationId(value)
    }
}

pub struct LocationWithId {
    pub id: LocationId,
}
