use crate::actor::Actor;
use async_trait::async_trait;

pub struct Strategy {
    pub name: String,
    pub is_active: bool,
}

#[async_trait]
pub trait ListAllNotificationStrategiesInteractor {
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<Strategy>>;
}
