use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use project_service::ServiceConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = ServiceConfig::from_env();
    tracing::info!(
        bind_addr = %config.bind_addr,
        timeout_secs = config.request_timeout_secs,
        "starting cargo-project-serve"
    );

    project_service::serve(config).await
}

/// Initialise the `tracing` subscriber.
///
/// Log level is controlled by the `RUST_LOG` environment variable
/// (default: `info`).  JSON output is emitted when `LOG_FORMAT=json`.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
