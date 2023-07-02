use crate::producer::Producer;
use crate::tasks::text_search::search_locations_by_text;
use crate::utils::progress_tracking::{get_progress_status, set_progress_status, TaskStatus};
use anyhow::Context;

use location_search::contracts::text_search::TextSearcher;
use shared_kernel::location_ids::ExternalLocationId;
use std::str::FromStr;
use std::string::ToString;

#[derive(Copy, Clone)]
pub enum Status {
    Pending,
    Success,
    Failure,
    NotFound,
}

pub struct LocationApiResponse {
    pub id: ExternalLocationId,
    pub name: String,
    pub address: String,
}

pub struct LocationResponseWithStatus {
    pub responses: Vec<LocationApiResponse>,
    pub status: Status,
}

impl Producer {
    pub async fn search_for_location(
        &self,
        text: impl AsRef<str>,
    ) -> anyhow::Result<LocationResponseWithStatus> {
        let text_searcher = TextSearcher::new();
        let text = text.as_ref().to_owned();
        let cached_response = text_searcher.cache_search(text.clone()).await?;

        if let Some(response) = cached_response {
            let responses = response
                .into_iter()
                .map(|prediction| LocationApiResponse {
                    id: prediction.id.inner().into(),
                    name: prediction.name,
                    address: prediction.address,
                })
                .collect();
            return Ok(LocationResponseWithStatus {
                responses,
                status: Status::Success,
            });
        }

        let progress = get_progress_status::<String, _>(&text, |val| {
            val.map(|val| TaskStatus::from_str(&val).context("Invalid progress type"))
                .transpose()
        })
        .await?;

        if let Some(progress) = progress {
            // for Success or Pending, just return Pending,
            if matches!(progress, TaskStatus::Pending | TaskStatus::Success) {
                return Ok(LocationResponseWithStatus {
                    responses: Default::default(),
                    status: Status::Pending,
                });
            }

            return Ok(LocationResponseWithStatus {
                responses: Default::default(),
                status: progress.into(),
            });
        }

        let status = set_progress_status(&text, TaskStatus::Pending.to_string(), |val| {
            TaskStatus::from_str(&val).context("Invalid progress type")
        })
        .await?;

        self.app
            .send_task(search_locations_by_text::new(text))
            .await
            .context("Failed to send task")?;

        Ok(LocationResponseWithStatus {
            responses: Default::default(),
            status: status.into(),
        })
    }
}
