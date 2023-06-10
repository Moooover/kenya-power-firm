use location_subscription::contracts::import_locations_to_search_engine::ImportLocationsToSearchEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_kernel::tracing::config_telemetry();
    start().await?;
    shared_kernel::tracing::shutdown_global_tracer_provider();
    Ok(())
}

async fn start() -> anyhow::Result<()> {
    ImportLocationsToSearchEngine::execute().await
}
