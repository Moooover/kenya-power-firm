use shared_kernel::{string_key, uuid_key};
use std::collections::HashMap;

string_key!(ExternalLocationId);

uuid_key!(LocationId);

// TODO: Refactor this, should be moved to the sqlx crate, maybe
#[derive(Clone)]
pub struct LocationInput {
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}
