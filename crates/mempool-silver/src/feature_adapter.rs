use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use chrono::{DateTime, Utc};

use anyhow::{Context, Result};
use mempool_core::{
    aggregate_bar_flags, ExchangeRegistry, FeeSnapshot, MempoolFeatureBar, MempoolTxEvent,
    SCHEMA_VERSION,
};
use mempool_core::typed_parquet::{read_fee_snapshots_typed, write_feature_bars_typed};

pub fn read_snapshots_jsonl(path: impl AsRef<Path>) -> Result<Vec<FeeSnapshot>> {
    let file = File::open(path.as_ref()).context("open fee snapshot jsonl")?;
    let mut out = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.context("read snapshot line")?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line).context("parse snapshot")?);
    }
    Ok(out)
}

pub fn read_snapshots_parquet(path: impl AsRef<Path>) -> Result<Vec<FeeSnapshot>> {
    read_fee_snapshots_typed(path).context("read typed fee snapshots parquet")
}

pub fn read_snapshots_from_run_dir(run_dir: impl AsRef<Path>) -> Result<Vec<FeeSnapshot>> {
    let run_dir = run_dir.as_ref();
    let mut snaps = Vec::new();
    if !run_dir.exists() {
        return Ok(snaps);
    }
    collect_fee_parts(run_dir, &mut snaps)?;
    snaps.sort_by(|a, b| a.observed_at.cmp(&b.observed_at));
    Ok(snaps)
}

fn collect_fee_parts(dir: &Path, out: &mut Vec<FeeSnapshot>) -> Result<()> {
    for entry in fs::read_dir(dir).context("read run dir")? {
        let entry = entry.context("dir entry")?;
        let path = entry.path();
        if path.is_dir() {
            collect_fee_parts(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("parquet") {
            out.extend(read_fee_snapshots_typed(&path).context("read fee part")?);
        }
    }
    Ok(())
}

pub fn read_tx_events_from_run_dir(run_dir: impl AsRef<Path>) -> Result<Vec<MempoolTxEvent>> {
    let run_dir = run_dir.as_ref();
    let mut events = Vec::new();
    if !run_dir.exists() {
        return Ok(events);
    }
    collect_tx_event_chunks(run_dir, &mut events)?;
    events.sort_by_key(|e| e.observed_at_ns);
    Ok(events)
}

fn collect_tx_event_chunks(dir: &Path, out: &mut Vec<MempoolTxEvent>) -> Result<()> {
    for entry in fs::read_dir(dir).context("read tx event run dir")? {
        let entry = entry.context("dir entry")?;
        let path = entry.path();
        if path.is_dir() {
            collect_tx_event_chunks(&path, out)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e == "gz")
        {
            out.extend(read_tx_events_gz(&path).context("read tx event chunk")?);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            out.extend(read_tx_events_jsonl(&path).context("read tx event jsonl")?);
        }
    }
    Ok(())
}

fn read_tx_events_gz(path: &Path) -> Result<Vec<MempoolTxEvent>> {
    let file = File::open(path).context("open tx event gzip")?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut out = Vec::new();
    for line in BufReader::new(decoder).lines() {
        let line = line.context("read gz line")?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line).context("parse tx event")?);
    }
    Ok(out)
}

pub fn read_tx_events_jsonl(path: impl AsRef<Path>) -> Result<Vec<MempoolTxEvent>> {
    let file = File::open(path.as_ref()).context("open tx events jsonl")?;
    let mut out = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.context("read tx line")?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line).context("parse tx event")?);
    }
    Ok(out)
}

pub fn build_feature_bars(
    snaps: &[FeeSnapshot],
    events: &[MempoolTxEvent],
    registry: &ExchangeRegistry,
    clearance_fee_sat_vb: f64,
) -> Vec<MempoolFeatureBar> {
    if snaps.is_empty() {
        return Vec::new();
    }
    let mut bars = Vec::with_capacity(snaps.len());
    let mut history: Vec<f64> = Vec::new();
    for s in snaps {
        history.push(s.p99_fee_sat_vb);
        let mean = history.iter().sum::<f64>() / history.len() as f64;
        let var = if history.len() <= 1 {
            0.0
        } else {
            let m = mean;
            history
                .iter()
                .map(|x| {
                    let d = x - m;
                    d * d
                })
                .sum::<f64>()
                / (history.len() as f64 - 1.0)
        };
        let std = var.sqrt();
        let z = if std > 0.0 {
            (s.p99_fee_sat_vb - mean) / std
        } else {
            0.0
        };
        let regime = if s.stuck_flow_pct >= 70.0 {
            "congested"
        } else if s.stuck_flow_pct <= 20.0 {
            "clearing"
        } else {
            "normal"
        };
        let window_events = events_for_snapshot(s, events);
        let (exchange_inflow_event, cpfp_detected) =
            aggregate_bar_flags(&window_events, registry, clearance_fee_sat_vb);
        bars.push(MempoolFeatureBar {
            schema_version: SCHEMA_VERSION.to_string(),
            ts_minute: s.observed_at.clone(),
            tx_count: s.size_txs,
            avg_fee_sat_vb: (s.p50_fee_sat_vb + s.p90_fee_sat_vb + s.p99_fee_sat_vb) / 3.0,
            p90_fee_sat_vb: s.p90_fee_sat_vb,
            p99_fee_sat_vb: s.p99_fee_sat_vb,
            stuck_flow_pct: s.stuck_flow_pct,
            fee_spike_zscore: z,
            exchange_inflow_event,
            cpfp_detected,
            congestion_regime: regime.to_string(),
        });
    }
    bars
}

pub fn write_features_parquet(path: impl AsRef<Path>, bars: &[MempoolFeatureBar]) -> Result<()> {
    write_feature_bars_typed(path, bars).context("write typed features parquet")
}

pub fn write_features_jsonl(path: impl AsRef<Path>, bars: &[MempoolFeatureBar]) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut f = File::create(path).context("create features output")?;
    for b in bars {
        let line = serde_json::to_string(b).context("serialize feature bar")?;
        writeln!(f, "{line}").context("write feature line")?;
    }
    Ok(())
}

fn events_for_snapshot(snap: &FeeSnapshot, events: &[MempoolTxEvent]) -> Vec<MempoolTxEvent> {
    let snap_ts = DateTime::parse_from_rfc3339(&snap.observed_at)
        .map(|d| d.with_timezone(&Utc).timestamp())
        .unwrap_or(0);
    events
        .iter()
        .filter(|e| {
            let ev_sec = e.observed_at_ns / 1_000_000_000;
            (ev_sec - snap_ts).abs() <= 60
        })
        .cloned()
        .collect()
}

pub fn load_registry() -> Result<ExchangeRegistry> {
    let path = std::env::var("HFT_EXCHANGE_REGISTRY")
        .unwrap_or_else(|_| "data/exchange_registry/v1/exchanges.json".to_string());
    ExchangeRegistry::load(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mempool_core::types::MempoolEntryMeta;
    use std::collections::HashSet;

    #[test]
    fn builds_feature_bars_with_cpfp_flag() {
        let snap_ts = DateTime::parse_from_rfc3339("2026-05-20T00:01:00Z")
            .unwrap()
            .with_timezone(&Utc)
            .timestamp();
        let snaps = vec![FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: "2026-05-20T00:01:00Z".into(),
            size_txs: 20,
            bytes: 200,
            min_fee_sat_vb: 2.0,
            p50_fee_sat_vb: 3.0,
            p90_fee_sat_vb: 6.0,
            p99_fee_sat_vb: 12.0,
            stuck_flow_pct: 80.0,
            source_node_id: "n1".into(),
        }];
        let events = vec![MempoolTxEvent {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at_ns: snap_ts * 1_000_000_000,
            txid: "tx1".into(),
            first_seen_at_ns: 0,
            fee_rate_sat_vb: 25.0,
            vsize: 200,
            rbf_signaling: false,
            output_values_sats: vec![],
            output_addresses: vec![],
            mempool_entry: Some(MempoolEntryMeta {
                ancestor_count: 1,
                descendant_count: 1,
                ancestor_feerate_sat_vb: Some(2.0),
                descendant_feerate_sat_vb: Some(25.0),
            }),
            node_sync_height: 0,
            node_ibd_complete: true,
            source_node_id: "n1".into(),
        }];
        let reg = ExchangeRegistry::from_addresses(HashSet::new());
        let bars = build_feature_bars(&snaps, &events, &reg, 10.0);
        assert!(bars[0].cpfp_detected);
        assert_eq!(bars[0].congestion_regime, "congested");
    }
}
