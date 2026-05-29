use anyhow::Result;
use mempool_core::IngestConfig;
use mempool_ingest::IngestRunner;
use tracing_subscriber::EnvFilter;

fn parse_cycles_arg() -> Result<Option<usize>> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--cycles" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("--cycles requires a numeric value"))?;
            let cycles = value
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("invalid --cycles value: {value}"))?;
            return Ok(Some(cycles));
        }
    }
    Ok(None)
}

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
    let cli_cycles = parse_cycles_arg()?;
    let env_cycles = std::env::var("HFT_MAX_POLL_CYCLES")
        .ok()
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("invalid HFT_MAX_POLL_CYCLES value: {value}"))
        })
        .transpose()?;
    let cycles = cli_cycles.or(env_cycles);
    match cycles {
        Some(n) => {
            tracing::info!(cycles = n, "running bounded ingest cycles");
            runner.run_for(n).await
        }
        None => runner.run().await,
    }
}
