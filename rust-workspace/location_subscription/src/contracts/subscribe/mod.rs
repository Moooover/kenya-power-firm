mod db_access;

use shared_kernel::location_ids::ExternalLocationId;

use crate::contracts::subscribe::db_access::SubscriptionDbAccess;
use crate::data_transfer::{
    AffectedSubscriber, AffectedSubscriberWithLocationMatchedAndLineSchedule,
    LocationMatchedAndLineSchedule,
};

use shared_kernel::subscriber_id::SubscriberId;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SubscribeToLocationError {
    #[error("Internal error")]
    InternalError(#[from] anyhow::Error),
    #[error("Expected error: {0}")]
    ExpectedError(String),
}

pub struct SubscribeInteractor {
    db: SubscriptionDbAccess,
}

impl Default for SubscribeInteractor {
    fn default() -> Self {
        Self::new()
    }
}

impl SubscribeInteractor {
    pub fn new() -> Self {
        Self {
            db: SubscriptionDbAccess::new(),
        }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn subscribe_to_location(
        &self,
        subscriber_id: SubscriberId,
        external_id: ExternalLocationId,
    ) -> Result<
        Option<AffectedSubscriberWithLocationMatchedAndLineSchedule>,
        SubscribeToLocationError,
    > {
        let existing_location = self
            .db
            .find_location_by_external_id(external_id.clone())
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        let location = match existing_location {
            None => main_location_search_and_save::execute(external_id, &self.db).await?,
            Some(location) => location,
        };

        let location_id = location.location_id;

        self.db
            .subscribe(subscriber_id, location_id)
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        let _ = nearby_locations_search_and_save::execute(&location, &self.db).await?;

        let affected_location = self
            .db
            .is_location_affected(location_id)
            .await
            .map_err(SubscribeToLocationError::InternalError)?;

        let result = affected_location.map(|data| {
            let affected_subscriber = match data.is_directly_affected {
                true => AffectedSubscriber::DirectlyAffected(subscriber_id),
                false => AffectedSubscriber::PotentiallyAffected(subscriber_id),
            };
            AffectedSubscriberWithLocationMatchedAndLineSchedule {
                affected_subscriber,
                location_matched: LocationMatchedAndLineSchedule {
                    line_schedule: data.line_matched,
                    location_id,
                    location_name: location.name,
                },
            }
        });

        Ok(result)
    }
}

mod search_utils {
    use serde::Deserialize;
    use serde::Serialize;
    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum StatusCode {
        OK,
        #[serde(rename = "ZERO_RESULTS")]
        Zeroresults,
        #[serde(rename = "INVALID_REQUEST")]
        InvalidRequest,
        #[serde(rename = "OVER_QUERY_LIMIT")]
        OverQueryLimit,
        #[serde(rename = "REQUEST_DENIED")]
        RequestDenied,
        #[serde(rename = "UNKNOWN_ERROR")]
        UnknownError,
    }

    impl StatusCode {
        pub fn is_cacheable(&self) -> bool {
            matches!(self, StatusCode::OK | StatusCode::Zeroresults)
        }
    }
}
mod main_location_search_and_save {
    use crate::config::SETTINGS_CONFIG;
    use crate::contracts::subscribe::db_access::SubscriptionDbAccess;
    use crate::contracts::subscribe::search_utils::StatusCode;
    use crate::save_and_search_for_locations::{LocationInput, LocationWithCoordinates};
    use anyhow::{anyhow, bail, Context};
    use secrecy::ExposeSecret;
    use serde::Deserialize;
    use shared_kernel::http_client::HttpClient;
    use shared_kernel::location_ids::ExternalLocationId;
    use url::Url;

    fn generate_url(id: ExternalLocationId) -> anyhow::Result<Url> {
        let place_details_path = "/place/details/json";

        let host = &SETTINGS_CONFIG.location.host;
        Url::parse_with_params(
            &format!("{}{}", host, place_details_path),
            &[
                ("key", SETTINGS_CONFIG.location.api_key.expose_secret()),
                ("place_id", &id.inner()),
            ],
        )
        .context("Failed to parse Url")
    }

    async fn get_place_details(url: Url) -> anyhow::Result<LocationInput> {
        #[derive(Deserialize, Debug, Clone)]
        struct ResponseResult {
            name: String,
            formatted_address: String,
            place_id: String,
        }
        #[derive(Deserialize, Debug, Clone)]
        struct Response {
            result: Option<ResponseResult>,
            status: StatusCode,
        }
        let result = HttpClient::get_json::<serde_json::Value>(url).await?;
        let response: Response = serde_json::from_value(result.clone())
            .with_context(|| format!("Invalid response {result:?}"))?;

        if response.status.is_cacheable() {
            if let Some(response_result) = response.result {
                return Ok(LocationInput {
                    external_id: ExternalLocationId::new(response_result.place_id),
                    name: response_result.name,
                    address: response_result.formatted_address,
                    api_response: result,
                });
            }
        }
        bail!("Failed to get valid response {result:?}")
    }

    async fn save_location_returning_id_and_coordinates(
        location: LocationInput,
        db: &SubscriptionDbAccess,
    ) -> anyhow::Result<LocationWithCoordinates> {
        let external_id = location.external_id.clone();
        let _ = db.save_main_location(location).await?;
        let location_id = db.find_location_by_external_id(external_id).await?;
        location_id
            .ok_or("Location not found")
            .map_err(|err| anyhow!(err))
    }

    pub(super) async fn execute(
        id: ExternalLocationId,
        db: &SubscriptionDbAccess,
    ) -> anyhow::Result<LocationWithCoordinates> {
        let url = generate_url(id)?;
        let location = get_place_details(url).await?;
        save_location_returning_id_and_coordinates(location, db).await
    }
}

mod nearby_locations_search_and_save {
    use crate::config::SETTINGS_CONFIG;
    use crate::contracts::subscribe::db_access::SubscriptionDbAccess;
    use crate::save_and_search_for_locations::{LocationWithCoordinates, NearbyLocationId};
    use anyhow::{bail, Context};

    use crate::contracts::subscribe::search_utils::StatusCode;
    use secrecy::ExposeSecret;
    use serde::Deserialize;
    use shared_kernel::http_client::HttpClient;
    use url::Url;

    fn generate_url(primary_location: &LocationWithCoordinates) -> anyhow::Result<Url> {
        let nearby_locations_path = "/place/nearbysearch/json?rankby=distance";
        let host = &SETTINGS_CONFIG.location.host;
        Url::parse_with_params(
            &format!("{}{}", host, nearby_locations_path),
            &[
                (
                    "location",
                    &format!(
                        "{} {}",
                        primary_location.latitude, primary_location.longitude
                    ),
                ),
                ("key", SETTINGS_CONFIG.location.api_key.expose_secret()),
            ],
        )
        .context("Failed to parse nearby_location URL")
    }

    async fn get_nearby_locations_from_api(url: Url) -> anyhow::Result<serde_json::Value> {
        let raw_response = HttpClient::get_json::<serde_json::Value>(url).await?;

        #[derive(Deserialize, Debug, Clone)]
        struct Response {
            status: StatusCode,
        }

        let response: Response = serde_json::from_value(raw_response.clone())
            .with_context(|| format!("Invalid response {raw_response:?}"))?;

        if response.status.is_cacheable() {
            return Ok(raw_response);
        }
        bail!("Failed to get valid response {raw_response:?}")
    }

    pub(super) async fn execute(
        primary_location: &LocationWithCoordinates,
        db: &SubscriptionDbAccess,
    ) -> anyhow::Result<NearbyLocationId> {
        let already_saved = db
            .are_nearby_locations_already_saved(primary_location.location_id)
            .await?;
        if let Some(already_saved) = already_saved {
            return Ok(already_saved);
        }
        let url = generate_url(primary_location)?;
        let api_response = get_nearby_locations_from_api(url.clone()).await?;
        db.save_nearby_locations(url, primary_location.location_id, api_response)
            .await
    }
}
