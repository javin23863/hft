use anyhow::Result;
use mempool_silver::{
    build_feature_bars, load_registry, read_snapshots_from_run_dir, read_tx_events_from_run_dir,
    read_tx_events_jsonl, write_features_parquet,
};

fn main() -> Result<()> {
    let run_date = std::env::var("HFT_RUN_DATE").unwrap_or_else(|_| "latest".to_string());
    let in_dir = format!(
        "runtime/spool/hft/bronze/source=bitcoin/dataset=mempool_fee_snapshot/run={run_date}"
    );
    let out_path = format!(
        "runtime/spool/hft/silver/source=bitcoin/dataset=mempool_features_1m/run={run_date}/features.parquet"
    );
    let clearance = std::env::var("HFT_CLEARANCE_FEE_SAT_VB")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10.0);
    let snaps = read_snapshots_from_run_dir(&in_dir)?;
    let events = if let Ok(path) = std::env::var("HFT_TX_EVENTS_JSONL") {
        read_tx_events_jsonl(path)?
    } else {
        let events_dir = format!(
            "runtime/spool/hft/bronze/source=bitcoin/dataset=mempool_tx_event/run={run_date}"
        );
        read_tx_events_from_run_dir(&events_dir)?
    };
    let registry = load_registry()?;
    let bars = build_feature_bars(&snaps, &events, &registry, clearance);
    write_features_parquet(out_path, &bars)?;
    println!("wrote {} rows", bars.len());
    Ok(())
}

