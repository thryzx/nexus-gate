use anyhow::Result;
use nexus_gate::{config::AppConfig, server};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::load()?;

    let _subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("nexus_gate={}", config.logging.level).into()),
        )
        .json()
        .flatten_event(true)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "starting nexus-gate"
    );

    server::run(config).await
}
