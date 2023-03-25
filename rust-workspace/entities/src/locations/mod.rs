use shared_kernel::{non_empty_string, string_key, uuid_key};

string_key!(ExternalLocationId);

uuid_key!(LocationId);

string_key!(LocationName);

#[derive(Clone, PartialEq, Eq, Hash)]
//TODO: Also check locations without details from the external apis
pub struct LocationDetails {
    pub id: LocationId,
    pub name: LocationName,
}
