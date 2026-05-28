use std::fs::File;
use std::io::Write;

use anyhow::Result;
use feasibility::{
    build_report, export_h1_observations, export_h2_observations, export_h3_observations,
    write_report_markdown,
};
use mempool_silver::{
    build_feature_bars, load_registry, read_snapshots_from_run_dir, read_tx_events_from_run_dir,
    read_tx_events_jsonl,
};

fn write_jsonl<T: serde::Serialize>(path: &str, rows: &[T]) -> Result<()> {
    let mut f = File::create(path)?;
    for row in rows {
        writeln!(f, "{}", serde_json::to_string(row)?)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let run_date = std::env::var("HFT_RUN_DATE").unwrap_or_else(|_| "latest".to_string());
    let bronze_run = format!(
        "runtime/spool/hft/bronze/source=bitcoin/dataset=mempool_fee_snapshot/run={run_date}"
    );
    let clearance = std::env::var("HFT_CLEARANCE_FEE_SAT_VB")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10.0);

    let registry = load_registry()?;
    let snaps = read_snapshots_from_run_dir(&bronze_run)?;
    let events = if let Ok(path) = std::env::var("HFT_TX_EVENTS_JSONL") {
        read_tx_events_jsonl(path)?
    } else {
        let events_dir = format!(
            "runtime/spool/hft/bronze/source=bitcoin/dataset=mempool_tx_event/run={run_date}"
        );
        read_tx_events_from_run_dir(&events_dir)?
    };
    let bars = build_feature_bars(&snaps, &events, &registry, clearance);

    let h1 = export_h1_observations(&events, &registry);
    let h2 = export_h2_observations(&events, clearance);
    let h3 = export_h3_observations(&bars);

    std::fs::create_dir_all("data/feasibility")?;
    write_jsonl("data/feasibility/h1_observations.jsonl", &h1)?;
    write_jsonl("data/feasibility/h2_observations.jsonl", &h2)?;
    write_jsonl("data/feasibility/h3_observations.jsonl", &h3)?;

    let report = build_report(&h1, &h2, &h3);
    write_report_markdown("docs/FEASIBILITY_REPORT.md", &report)?;
    println!(
        "exported feasibility observations (h1={}, h2={}, h3={})",
        h1.len(),
        h2.len(),
        h3.len()
    );
    Ok(())
}
