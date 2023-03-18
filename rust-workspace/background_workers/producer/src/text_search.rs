use anyhow::Context;
use async_trait::async_trait;
use tasks::{
    configuration::REPO,
    text_search::{generate_search_url, search_locations_by_text},
};
use use_cases::search_for_locations::{LocationApiResponse, LocationSearchApi};

use crate::producer::Producer;

// fn get_search_results() {
//     // check cache
//     // if not present in cache
//     // check status
//     // if not present
//     // call worker thread
//     // worker thread first checks cache
//     // then gets a token before calling the api
//     // once apiresponse is saved, change state to resolved
//     // etc..
// }

#[async_trait]
impl LocationSearchApi for Producer {
    async fn search(&self, text: String) -> anyhow::Result<Vec<LocationApiResponse>> {
        let repository = REPO.get().await;
        let url = generate_search_url(text.clone())?;
        let cached_response = repository.get_cached_text_search_response(&url).await?;
        if let Some(response) = cached_response {
            return Ok(response
                .predictions
                .into_iter()
                .map(|prediction| LocationApiResponse {
                    id: prediction.place_id.into(),
                    name: prediction.description,
                })
                .collect());
        }

        self.app
            .send_task(search_locations_by_text::new(text))
            .await
            .context("Failed to send task")?;

        Ok(Default::default())
    }
}
