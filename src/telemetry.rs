use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Sync + Send {
    // Create log filter; try to read from RUST_LOG env variable and default to env_filter if
    // env var not set.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // Create a layer in the Bunyan format style outputting to stdout.
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);

    // Register our layers
    // JsonStorageLayer processes spans data and stores the associated metadata into JSON format for downstream layers.
    // BunyanFormattingLayer builds on top of JsonStorageLayer and creates bunyan-compatible JSON formatting.
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    // Redirect all `log`'s events to our subscriber; this allows us to tracing events from dependencies
    LogTracer::init().expect("Failed to set logger");
    // Set our subscriber to be the global default
    set_global_default(subscriber).expect("Failed to set subscriber");
}
