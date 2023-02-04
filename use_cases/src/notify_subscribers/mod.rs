use crate::import_planned_blackouts::{Area, ImportInput, SubscriberNotifier};
use async_trait::async_trait;
use power_interuptions::location::{AffectedArea, AreaId};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use subscription::subscriber::SubscriberId;

#[async_trait]
pub trait SubscriberRepo {
    async fn get_subscribers_from_affected_areas(
        &self,
        areas: &[AreaId],
    ) -> Result<HashMap<AreaId, Vec<SubscriberId>>, Box<dyn Error>>;
}

struct SubscriberNotifierImpl {
    repo: Arc<dyn SubscriberRepo>,
    notifier: Arc<dyn Notify>,
}

pub struct SubscriberNotification {
    subscriber_id: SubscriberId,
    areas_affected: Vec<AffectedArea>,
}

#[async_trait]
pub trait Notify {
    async fn notify(
        &self,
        notifications: Vec<SubscriberNotification>,
    ) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl SubscriberNotifier for SubscriberNotifierImpl {
    async fn send_notifications_to_subscribers(
        &self,
        data: Vec<AffectedArea>,
    ) -> Result<(), Box<dyn Error>> {
        let subscribers = self
            .repo
            .get_subscribers_from_affected_areas(&vec![])
            .await?;

        let _ = self.notifier.notify(vec![]).await?;
        todo!()
    }
}
