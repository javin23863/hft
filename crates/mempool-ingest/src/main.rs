use anyhow::Result;
use mempool_core::IngestConfig;
use mempool_ingest::IngestRunner;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = IngestConfig::from_env()?;
    tracing::info!(?cfg, "loaded ingest config");
    let mut runner = IngestRunner::new(cfg)?;
    runner.run().await
}
