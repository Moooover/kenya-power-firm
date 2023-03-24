use crate::producer::Producer;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use entities::subscriptions::SubscriberId;
use std::collections::HashMap;
use tasks::subscribe_to_location::fetch_and_subscribe_to_locations;
use use_cases::subscriber_locations::data::LocationInput;
use use_cases::subscriber_locations::subscribe_to_location::LocationSubscriber;

#[async_trait]
impl LocationSubscriber for Producer {
    async fn subscribe_to_location(
        &self,
        location: LocationInput<ExternalLocationId>,
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<()> {
        self.app
            .send_task(fetch_and_subscribe_to_locations::new(
                location.primary_id().to_owned(),
                location.nearby_locations,
                subscriber_id,
            ))
            .await
            .context("Failed to send task")?;

        Ok(())
    }
}
