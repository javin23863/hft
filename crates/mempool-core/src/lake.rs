//! B2 lake key helpers under the `hft/` prefix (hft-owned namespace).

/// Bronze JSONL chunk for per-tx mempool observations.
pub fn bronze_tx_event_key(run_date: &str, hour: &str, chunk_ms: u64) -> String {
    format!(
        "hft/bronze/source=bitcoin/dataset=mempool_tx_event/run={run_date}/{hour}/chunk_{chunk_ms}.jsonl.gz"
    )
}

/// Bronze parquet fee distribution snapshot.
pub fn bronze_fee_snapshot_key(run_date: &str) -> String {
    format!(
        "hft/bronze/source=bitcoin/dataset=mempool_fee_snapshot/run={run_date}/snapshot.parquet"
    )
}

/// Silver 1m causal feature bars.
pub fn silver_features_1m_key(run_date: &str) -> String {
    format!(
        "hft/silver/source=bitcoin/dataset=mempool_features_1m/run={run_date}/features_1m.parquet"
    )
}

/// Silver scenario hypothesis signals.
pub fn silver_scenario_signal_key(run_date: &str) -> String {
    format!("hft/silver/source=bitcoin/dataset=scenario_signal/run={run_date}/signals.parquet")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_use_hft_prefix() {
        let k = bronze_tx_event_key("2026-05-20", "14", 1_700_000_000_000);
        assert!(k.starts_with("hft/bronze/"));
        assert!(!k.starts_with("quantx/"));
    }
}
