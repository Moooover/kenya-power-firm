use async_trait::async_trait;
use entities::power_interruptions::location::FutureOrCurrentNairobiTZDateTime;
use itertools::Itertools;
use location_subscription::contracts::get_affected_subscribers_from_import::AffectedSubscribersInteractor;
use location_subscription::contracts::get_affected_subscribers_from_import::{
    Area, County, ImportInput, Region, TimeFrame,
};
use location_subscription::data_transfer::AffectedSubscriber;
use notifications::contracts::send_notification::AffectedSubscriber as NotificationAffectedSubscriber;
use notifications::contracts::send_notification::LocationMatchedAndLineSchedule as NotificationLocationMatchedAndLineSchedule;
use notifications::contracts::send_notification::{
    AffectedSubscriberWithLocations, LineWithScheduledInterruptionTime,
};
use pdf_text_parser::PDFContentExtractor;
use producer::producer::Producer;
use sqlx_postgres::repository::Repository;
use std::sync::Arc;
use use_cases::actor::{Actor, Permissions, SubscriberExternalId};
use use_cases::import_affected_areas::ImportPlannedBlackoutsInteractor;
use web_page_extractor::{pdf_extractor::PdfExtractorImpl, WebPageExtractor};

#[derive(Debug)]
struct ImportActor;

#[async_trait]
impl Actor for ImportActor {
    fn permissions(&self) -> Permissions {
        // TODO: Get the permissions from auth0;
        let permissions: Vec<String> = vec!["import:affected_regions".to_string()];

        permissions.as_slice().into()
    }

    fn external_id(&self) -> SubscriberExternalId {
        "MAIN_IMPORTER".to_string().try_into().unwrap()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_kernel::tracing::config_telemetry("importer");
    start().await?;
    shared_kernel::tracing::shutdown_global_tracer_provider();
    Ok(())
}

async fn start() -> anyhow::Result<()> {
    let repo = Repository::new().await?;

    let content_extractor = PDFContentExtractor;

    let content_extractor = Arc::new(content_extractor);

    let pdf_extractor = PdfExtractorImpl::new(content_extractor);

    let repo = Arc::new(repo);

    let producer = Producer::new().await?;

    let importer = use_cases::import_affected_areas::ImportAffectedAreas::new(repo.clone());

    let extractor = WebPageExtractor::new(repo, Arc::new(pdf_extractor));
    let import_input = extractor.run(&ImportActor {}).await?;
    importer
        .import(&ImportActor {}, import_input.clone())
        .await?;

    let data = import_input
        .0
        .iter()
        .map(|(url, imported_regions)| {
            let regions = imported_regions
                .iter()
                .map(|region| Region {
                    name: region.name.clone(),
                    counties: region
                        .counties
                        .iter()
                        .map(|county| County {
                            name: county.name.clone(),
                            areas: county
                                .areas
                                .iter()
                                .filter_map(|area| {
                                    let from = FutureOrCurrentNairobiTZDateTime::try_from(
                                        area.from.clone(),
                                    );
                                    let to =
                                        FutureOrCurrentNairobiTZDateTime::try_from(area.to.clone());
                                    from.and_then(|from| {
                                        to.map(|to| Area {
                                            name: area.name.clone(),
                                            time_frame: TimeFrame { from, to },
                                            locations: area.locations.clone(),
                                        })
                                    })
                                    .ok()
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect_vec();
            (url.clone(), regions)
        })
        .collect();

    let data =
        AffectedSubscribersInteractor::get_affected_subscribers_from_import(ImportInput(data))
            .await?;

    let affected_locations_with_subscribers = data
        .into_iter()
        .flat_map(|(affected_subscriber, locations)| {
            let subscriber = match affected_subscriber {
                AffectedSubscriber::DirectlyAffected(subscriber) => {
                    NotificationAffectedSubscriber::DirectlyAffected(subscriber)
                }
                AffectedSubscriber::PotentiallyAffected(subscriber) => {
                    NotificationAffectedSubscriber::PotentiallyAffected(subscriber)
                }
            };
            let split_locations = locations
                .into_iter()
                .into_group_map_by(|data| data.line_schedule.source_url.clone());

            split_locations.into_iter().map(move |(url, locations)| {
                AffectedSubscriberWithLocations {
                    source_url: url,
                    subscriber: subscriber.clone(),
                    locations: locations
                        .into_iter()
                        .map(|location| NotificationLocationMatchedAndLineSchedule {
                            line_schedule: LineWithScheduledInterruptionTime {
                                line_name: location.line_schedule.line_name,
                                from: location.line_schedule.from,
                                to: location.line_schedule.to,
                            },
                            location_id: location.location_id,
                        })
                        .collect_vec(),
                }
            })
        })
        .collect_vec();
    producer
        .send_notifications(affected_locations_with_subscribers)
        .await
}
