use import_scheduled_interruptions::contracts::import_interruptions::ImportInterruptions;
use itertools::Itertools;
use location_subscription::contracts::get_affected_subscribers_from_import::AffectedSubscribersInteractor;
use location_subscription::contracts::get_affected_subscribers_from_import::{
    Area, County, ImportInput, Region, TimeFrame,
};

use tasks::producer::Producer;

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

    let affected_locations_with_subscribers =
        import_and_notify_affected_subscribers::convert_data_to_producer_input(data);
    producer
        .send_notifications(affected_locations_with_subscribers)
        .await
}
