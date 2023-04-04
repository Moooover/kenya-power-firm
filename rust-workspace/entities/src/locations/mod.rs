use shared_kernel::{string_key, uuid_key};

string_key!(ExternalLocationId);

uuid_key!(LocationId);

string_key!(LocationName);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LocationDetails {
    pub id: LocationId,
    pub name: LocationName,
}
