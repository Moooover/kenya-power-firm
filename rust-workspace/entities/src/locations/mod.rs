use std::collections::HashMap;

pub struct ExternalLocationId(String);

impl AsRef<str> for ExternalLocationId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for ExternalLocationId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

pub struct LocationInput {
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}
