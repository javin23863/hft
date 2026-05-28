use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use mempool_core::parquet_io::{read_json_rows_parquet, write_json_rows_parquet};
use mempool_core::{FeeSnapshot, MempoolFeatureBar, SCHEMA_VERSION};

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
    read_json_rows_parquet(path).context("read fee snapshots parquet")
}

pub fn build_feature_bars(snaps: &[FeeSnapshot]) -> Vec<MempoolFeatureBar> {
    if snaps.is_empty() {
        return Vec::new();
    }
    let mut bars = Vec::with_capacity(snaps.len());
    let mut history: Vec<f64> = Vec::new();
    for s in snaps {
        history.push(s.p99_fee_sat_vb);
        let mean = if history.is_empty() {
            0.0
        } else {
            history.iter().sum::<f64>() / history.len() as f64
        };
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
        bars.push(MempoolFeatureBar {
            schema_version: SCHEMA_VERSION.to_string(),
            ts_minute: s.observed_at.clone(),
            tx_count: s.size_txs,
            avg_fee_sat_vb: (s.p50_fee_sat_vb + s.p90_fee_sat_vb + s.p99_fee_sat_vb) / 3.0,
            p90_fee_sat_vb: s.p90_fee_sat_vb,
            p99_fee_sat_vb: s.p99_fee_sat_vb,
            stuck_flow_pct: s.stuck_flow_pct,
            fee_spike_zscore: z,
            exchange_inflow_event: false,
            cpfp_detected: false,
            congestion_regime: regime.to_string(),
        });
    }
    bars
}

pub fn write_features_parquet(path: impl AsRef<Path>, bars: &[MempoolFeatureBar]) -> Result<()> {
    write_json_rows_parquet(path, bars).context("write features parquet")
}

pub fn write_features_jsonl(path: impl AsRef<Path>, bars: &[MempoolFeatureBar]) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut f = File::create(path).context("create features output")?;
    for b in bars {
        let line = serde_json::to_string(b).context("serialize feature bar")?;
        use std::io::Write;
        writeln!(f, "{line}").context("write feature line")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_feature_bars() {
        let snaps = vec![
            FeeSnapshot {
                schema_version: SCHEMA_VERSION.to_string(),
                observed_at: "2026-05-20T00:00:00Z".into(),
                size_txs: 10,
                bytes: 100,
                min_fee_sat_vb: 1.0,
                p50_fee_sat_vb: 2.0,
                p90_fee_sat_vb: 5.0,
                p99_fee_sat_vb: 10.0,
                stuck_flow_pct: 10.0,
                source_node_id: "n1".into(),
            },
            FeeSnapshot {
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
            },
        ];
        let bars = build_feature_bars(&snaps);
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[1].congestion_regime, "congested");
    }
}
