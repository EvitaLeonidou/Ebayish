use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

//impl Subscriber for simplicity it returns subscriber
//+ Send + Sync needed for init_subscriber
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    //all logs in info level or above unless RUST_LOG says otherwish
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

//register a subscriber as a global default to process span data
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    //redirect all log to subscriber
    LogTracer::init().expect("Failed to set logger");

    set_global_default(subscriber).expect("Failed to set subscriber");
}