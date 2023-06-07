mod db_access;

use crate::contracts::send_notification::email::db_access::EmailNotificationsDbAccess;
use crate::contracts::send_notification::AffectedSubscriberWithLocations;

pub struct EmailNotificationInteractor;

impl EmailNotificationInteractor {
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn send(
        &self,
        subscriber_with_locations: AffectedSubscriberWithLocations,
    ) -> anyhow::Result<()> {
        let db = EmailNotificationsDbAccess::new();
        let email =
            email_notification::EmailNotification::generate(&db, subscriber_with_locations).await?;
        if let Some(notification) = email {
            return notification.send(&db).await;
        }
        Ok(())
    }
}

mod email_notification {
    use crate::contracts::send_notification::db_access::Notification;
    use crate::contracts::send_notification::email::db_access::EmailNotificationsDbAccess;
    use crate::contracts::send_notification::email::email_notification_sender;
    use crate::contracts::send_notification::{
        AffectedSubscriber, AffectedSubscriberWithLocations, LocationMatchedAndLineSchedule,
    };
    use crate::db_access::DbNotificationIdempotencyKey;
    use entities::locations::LocationId;
    use entities::subscriptions::SubscriberId;
    use itertools::Itertools;
    use std::collections::{HashMap, HashSet};
    use url::Url;

    #[derive(Clone, Debug)]
    pub struct EmailNotification(AffectedSubscriberWithLocations);

    impl Notification for EmailNotification {
        fn subscriber(&self) -> AffectedSubscriber {
            self.0.subscriber.clone()
        }

        fn locations_matched(&self) -> Vec<LocationMatchedAndLineSchedule> {
            self.0.locations.clone()
        }

        fn url(&self) -> Url {
            self.0.source_url.clone()
        }
    }

    impl EmailNotification {
        pub(crate) fn subscriber_id(&self) -> SubscriberId {
            self.0.subscriber.id()
        }
        pub(crate) fn location_ids(&self) -> HashSet<LocationId> {
            self.0
                .locations
                .iter()
                .map(|location| location.location_id)
                .collect()
        }

        #[tracing::instrument(skip(db_access), level = "debug")]
        pub(super) async fn generate(
            db_access: &EmailNotificationsDbAccess,
            data: AffectedSubscriberWithLocations,
        ) -> anyhow::Result<Option<Self>> {
            let email_strategy_id = db_access.get_email_strategy_id().await?.inner();
            let source_id = db_access.get_source_by_url(&data.source_url).await?;
            let subscriber_id = data.subscriber.id();
            let mapping_of_idempotency_key_to_affected_location = data
                .locations
                .iter()
                .map(|data| {
                    (
                        DbNotificationIdempotencyKey {
                            source_id: source_id.inner(),
                            subscriber_id: subscriber_id.inner(),
                            line: data.line_schedule.line_name.clone(),
                            strategy_id: email_strategy_id,
                        },
                        data.clone(),
                    )
                })
                .collect::<HashMap<_, _>>();

            let keys = mapping_of_idempotency_key_to_affected_location
                .keys()
                .cloned()
                .collect::<HashSet<_>>();

            let lines = data
                .locations
                .iter()
                .map(|data| data.line_schedule.line_name.clone())
                .collect_vec();
            let already_sent = DbNotificationIdempotencyKey::get_already_send_notifications(
                db_access,
                email_strategy_id,
                subscriber_id,
                lines,
                source_id,
            )
            .await?;

            let difference = keys
                .difference(&already_sent)
                .into_iter()
                .filter_map(|key| {
                    mapping_of_idempotency_key_to_affected_location
                        .get(key)
                        .cloned()
                })
                .collect_vec();

            if difference.is_empty() {
                return Ok(None);
            }

            Ok(Some(EmailNotification(AffectedSubscriberWithLocations {
                locations: difference,
                ..data
            })))
        }
        #[tracing::instrument(skip(self, db_access), level = "debug")]
        pub(super) async fn send(
            &self,
            db_access: &EmailNotificationsDbAccess,
        ) -> anyhow::Result<()> {
            email_notification_sender::send(self.clone(), db_access).await
        }
    }
}

mod email_notification_sender {
    use crate::config::SETTINGS_CONFIG;
    use crate::contracts::send_notification::db_access::Notification;
    use crate::contracts::send_notification::email::db_access::EmailNotificationsDbAccess;
    use crate::contracts::send_notification::email::email_notification::EmailNotification;
    use crate::contracts::send_notification::{AffectedSubscriber, LocationMatchedAndLineSchedule};
    use anyhow::Context;
    use secrecy::ExposeSecret;
    use serde::Deserialize;
    use serde::Serialize;
    use shared_kernel::http_client::HttpClient;
    use std::collections::HashMap;
    use url::Url;

    #[derive(Serialize, Deserialize)]
    enum AffectedState {
        #[serde(rename = "directly affected")]
        DirectlyAffected,
        #[serde(rename = "potentially affected")]
        PotentiallyAffected,
    }

    impl From<AffectedSubscriber> for AffectedState {
        fn from(subscriber: AffectedSubscriber) -> Self {
            match subscriber {
                AffectedSubscriber::DirectlyAffected(_) => Self::DirectlyAffected,
                AffectedSubscriber::PotentiallyAffected(_) => Self::PotentiallyAffected,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct AffectedLocation {
        pub location: String,
        pub date: String,
        pub start_time: String,
        pub end_time: String,
    }

    impl AffectedLocation {
        fn generate(data: &LocationMatchedAndLineSchedule, location: String) -> Self {
            let date_in_nairobi_time = data.line_schedule.from.to_date_time();
            let date = date_in_nairobi_time.format("%d/%m/%Y");
            let start_time = date_in_nairobi_time.format("%H:%M");
            let end_time = data.line_schedule.to.to_date_time().format("%H:%M");
            Self {
                location,
                date: date.to_string(),
                start_time: start_time.to_string(),
                end_time: end_time.to_string(),
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct MessageData {
        pub recipient_name: String,
        pub affected_state: AffectedState,
        pub link: String,
        pub affected_locations: Vec<AffectedLocation>,
    }

    #[derive(Serialize, Deserialize)]
    struct To {
        pub email: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Message {
        pub to: To,
        pub template: String,
        pub data: MessageData,
    }

    #[derive(Serialize, Deserialize)]
    struct Data {
        pub message: Message,
    }

    #[tracing::instrument(skip(db), level = "debug")]
    pub async fn send(
        email: EmailNotification,
        db: &EmailNotificationsDbAccess,
    ) -> anyhow::Result<()> {
        let body = generate_email_body(&email, db).await?;

        let settings = &SETTINGS_CONFIG.email;
        let url = Url::parse(&settings.host)
            .with_context(|| format!("Invalid url {}", &settings.host))?;

        let auth_token = settings.auth_token.expose_secret();
        let bearer_token = format!("Bearer {auth_token}");

        let headers = HashMap::from([("Authorization", bearer_token)]);

        let body = serde_json::to_value(body)
            .with_context(|| "Failed to convert the body to a valid json")?;

        #[derive(Deserialize, Debug)]
        struct Response {
            #[serde(rename = "requestId")]
            request_id: String,
        }

        let response = HttpClient::post_json::<Response>(url, headers, Some(body)).await?;
        db.save_email_notification_sent(email, response.request_id)
            .await
    }

    #[tracing::instrument(skip(db), level = "debug")]
    async fn generate_email_body(
        email: &EmailNotification,
        db: &EmailNotificationsDbAccess,
    ) -> anyhow::Result<Data> {
        let subscriber = db
            .as_ref()
            .find_subscriber_by_id(email.subscriber_id())
            .await?;

        let locations = email.location_ids();

        let locations = db.as_ref().get_locations_by_ids(locations).await?;

        let affected_locations = email
            .locations_matched()
            .iter()
            .filter_map(|affected_line| {
                locations
                    .get(&affected_line.location_id)
                    .map(|location_details| {
                        AffectedLocation::generate(affected_line, location_details.name.to_string())
                    })
            })
            .collect::<Vec<_>>();

        let message = Data {
            message: Message {
                to: To {
                    email: subscriber.email.to_string(),
                },
                template: SETTINGS_CONFIG.email.template_id.clone(),
                data: MessageData {
                    recipient_name: subscriber.name.to_string(),
                    affected_state: email.subscriber().into(),
                    link: email.url().to_string(),
                    affected_locations,
                },
            },
        };

        Ok(message)
    }
}
