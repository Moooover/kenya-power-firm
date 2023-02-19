use anyhow::anyhow;
use std::collections::HashSet;
use subscriptions::subscriber::SubscriberId;
use uuid::Uuid;

pub struct Permissions {
    permissions: HashSet<Permission>,
}

impl Permissions {
    fn contains(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum Permission {
    ImportAffectedAreas,
}
pub trait Actor: Send + Sync {
    fn permissions(&self) -> Permissions;

    fn id(&self) -> SubscriberId;

    fn check_for_permission(&self, permission: Permission) -> anyhow::Result<()> {
        match self.permissions().contains(permission) {
            true => Ok(()),
            false => Err(anyhow!("Unauthorized")),
        }
    }
}
