use location_subscription::contracts::get_currently_affected_subscribers::CurrentlyAffectedSubscribersInteractor;
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

    let affected_subscribers_interactor = CurrentlyAffectedSubscribersInteractor::new();
    let affected_subscribers = affected_subscribers_interactor.get().await?;

    let affected_locations_with_subscribers =
        import_and_notify_affected_subscribers::convert_data_to_producer_input(
            affected_subscribers,
        );
    producer
        .send_notifications(affected_locations_with_subscribers)
        .await
}
