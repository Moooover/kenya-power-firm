use opentelemetry::global;
use opentelemetry::sdk::propagation::BaggagePropagator;
use opentelemetry::sdk::propagation::TextMapCompositePropagator;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace;
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

pub fn config_telemetry(service_name: &'static str) {
    // Needed to forward ordinary log statements to our tracing subscriber.
    tracing_log::LogTracer::init().expect("Failed to initialize log tracer");

    // Initialize `tracing` using `opentelemetry-tracing` and configure logging
    let subscriber = Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(true)
                .with_thread_names(true),
        );

    let otel_layer = std::env::var("SKIP_OTLP_EXPORTER").ok().map_or_else(
        || {
            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
                .with_trace_config(trace::config().with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name),
                ])))
                .install_batch(opentelemetry::runtime::TokioCurrentThread)
                .expect("Failed to initialize otlp tracer.");
            Some(tracing_opentelemetry::layer().with_tracer(tracer))
        },
        |_| {
            dbg!("Skipping OTLP");
            None
        },
    );

    tracing::subscriber::set_global_default(subscriber.with(otel_layer))
        .expect("Failed to install `tracing` subscriber");

    let baggage_propagator = BaggagePropagator::new();
    let trace_context_propagator = TraceContextPropagator::new();
    let composite_propagator = TextMapCompositePropagator::new(vec![
        Box::new(baggage_propagator),
        Box::new(trace_context_propagator),
    ]);
    global::set_text_map_propagator(composite_propagator);
}

pub fn shutdown_global_tracer_provider() {
    global::shutdown_tracer_provider();
}
