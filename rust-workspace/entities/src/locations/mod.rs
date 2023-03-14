use shared_kernel::string_key;
use std::collections::HashMap;

string_key!(ExternalLocationId);

pub struct LocationInput {
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}
