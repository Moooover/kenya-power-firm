use import_scheduled_interruptions::contracts::import_interruptions::ImportInterruptions;
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

use producer::producer::Producer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_kernel::tracing::config_telemetry();
    start().await?;
    shared_kernel::tracing::shutdown_global_tracer_provider();
    Ok(())
}

async fn start() -> anyhow::Result<()> {
    let producer = Producer::new().await?;

    let import_input = ImportInterruptions::import().await?;

    let data = import_input
        .iter()
        .map(|(url, imported_regions)| {
            let regions = imported_regions
                .iter()
                .map(|region| Region {
                    name: region.region.clone(),
                    counties: region
                        .counties
                        .iter()
                        .map(|county| County {
                            name: county.name.clone(),
                            areas: county
                                .areas
                                .iter()
                                .map(|area| Area {
                                    name: area.name.to_string(),
                                    time_frame: TimeFrame {
                                        from: area.time_frame.from.clone(),
                                        to: area.time_frame.to.clone(),
                                    },
                                    locations: area.locations.clone(),
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
