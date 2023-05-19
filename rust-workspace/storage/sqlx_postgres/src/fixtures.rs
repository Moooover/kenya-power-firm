use crate::repository::Repository;

use lazy_static::lazy_static;

use chrono::{Days, Utc};
use entities::subscriptions::details::{SubscriberDetails, SubscriberExternalId};
use std::collections::HashMap;
use url::Url;

use use_cases::authentication::SubscriberAuthenticationRepo;

use entities::power_interruptions::location::FutureOrCurrentNairobiTZDateTime;
use entities::power_interruptions::location::{
    Area, County, ImportInput, NairobiTZDateTime, Region, TimeFrame,
};
use use_cases::import_affected_areas::SaveBlackoutAffectedAreasRepo;
lazy_static! {
    pub static ref SUBSCRIBER_EXTERNAL_ID: SubscriberExternalId =
        "ChIJGdueTt0VLxgRk19ir6oE8I0".to_owned().try_into().unwrap();
    pub static ref TOMORROW: FutureOrCurrentNairobiTZDateTime = {
        let tomorrow = NairobiTZDateTime::try_from(
            Utc::now()
                .naive_utc()
                .checked_add_days(Days::new(1))
                .unwrap(),
        )
        .unwrap();
        let tomorrow: FutureOrCurrentNairobiTZDateTime = tomorrow.try_into().unwrap();
        tomorrow
    };
}

impl Repository {
    /// TODO: Remove this from here, will move the subscriber logic to its own sub-system.
    #[allow(dead_code)]
    pub(crate) async fn fixtures(&self) {
        self.create_subscriber().await;
        self.save(&generate_import_input()).await.unwrap();
    }
    #[allow(dead_code)]
    async fn create_subscriber(&self) {
        self.create_or_update_subscriber(SubscriberDetails {
            name: "Tev".to_owned().try_into().unwrap(),
            email: "tevinthuku@gmail.com".to_owned().try_into().unwrap(),
            external_id: SUBSCRIBER_EXTERNAL_ID.clone(),
        })
        .await
        .unwrap();
    }
}

fn garden_city_area() -> Area<FutureOrCurrentNairobiTZDateTime> {
    Area {
        name: "Garden City".to_string().into(),
        time_frame: TimeFrame {
            from: TOMORROW.clone(),
            to: TOMORROW.clone(),
        },
        locations: vec![
            "Will Mary Estate".to_string(),
            "Garden City Mall".to_string(),
        ],
    }
}

pub fn nairobi_region() -> Region {
    Region {
        region: "Nairobi".to_string(),
        counties: vec![County {
            name: "Nairobi".to_string(),
            areas: vec![
                garden_city_area(),
                Area {
                    name: "Kibera".to_string().into(),
                    time_frame: TimeFrame {
                        from: TOMORROW.clone(),
                        to: TOMORROW.clone(),
                    },
                    locations: vec!["Pentecostal church".to_string()],
                },
            ],
        }],
    }
}

/// TODO: Fix this in a separate PR
#[allow(dead_code)]
fn generate_import_input() -> ImportInput {
    let url = Url::parse("https://example.net").unwrap();
    ImportInput::new(HashMap::from([(url, vec![nairobi_region()])]))
}
