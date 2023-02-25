use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use subscriptions::subscriber::SubscriberId;
use uuid::Uuid;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Deserialize)]
pub enum Permission {
    #[serde(rename = "import:affected_areas")]
    ImportAffectedAreas,
}

pub struct Permissions {
    permissions: HashSet<Permission>,
}

impl Permissions {
    fn contains(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }
}

impl From<&[String]> for Permissions {
    fn from(value: &[String]) -> Self {
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum MaybePermission {
            Yes(Permission),
            No(serde_json::Value),
        }
        let json_string_array = serde_json::to_string(&value).unwrap_or_default();
        let permissions = serde_json::from_str::<Vec<MaybePermission>>(&json_string_array)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|maybe| match maybe {
                MaybePermission::Yes(p) => Some(p),
                MaybePermission::No(_) => None,
            })
            .collect();
        Self { permissions }
    }
}

#[derive(Debug)]
pub struct ExternalId(String);

impl From<String> for ExternalId {
    fn from(value: String) -> Self {
        ExternalId(value)
    }
}

pub trait Actor: Send + Sync {
    fn permissions(&self) -> Permissions;

    fn external_id(&self) -> ExternalId;

    fn check_for_permission(&self, permission: Permission) -> anyhow::Result<()> {
        match self.permissions().contains(permission) {
            true => Ok(()),
            false => Err(anyhow!("Unauthorized")),
        }
    }
}
