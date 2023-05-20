use anyhow::anyhow;
use entities::subscriptions::details::SubscriberExternalId as SubscriberExternalIdInner;
#[cfg(test)]
use mockall::automock;
use serde::Deserialize;

use std::collections::HashSet;
use std::fmt::Debug;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Deserialize)]
pub enum Permission {
    #[serde(rename = "import:affected_regions")]
    ImportAffectedRegions,
}

#[derive(Debug)]
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

impl ToString for ExternalId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for ExternalId {
    fn from(value: String) -> Self {
        ExternalId(value)
    }
}

#[derive(Debug, Clone)]
pub struct SubscriberExternalId(SubscriberExternalIdInner);

impl From<SubscriberExternalId> for SubscriberExternalIdInner {
    fn from(value: SubscriberExternalId) -> Self {
        value.0
    }
}

impl TryFrom<String> for SubscriberExternalId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = SubscriberExternalIdInner::try_from(value)?;
        Ok(SubscriberExternalId(value))
    }
}

#[cfg_attr(test, automock)]
pub trait Actor: Send + Sync + Debug {
    fn permissions(&self) -> Permissions;

    fn external_id(&self) -> SubscriberExternalId;

    fn check_for_permission(&self, permission: Permission) -> anyhow::Result<()> {
        match self.permissions().contains(permission) {
            true => Ok(()),
            false => Err(anyhow!("Unauthorized")),
        }
    }
}
