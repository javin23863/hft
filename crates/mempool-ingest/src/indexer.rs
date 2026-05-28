use mempool_core::schema::SCHEMA_VERSION;
use mempool_core::types::{FeeSnapshot, MempoolTxEvent};

use crate::rpc::{BtcRpcClient, VerboseMempoolEntry};

#[derive(Debug, Default)]
pub struct MempoolIndexer {
    fee_rates: Vec<f64>,
    seen_txids: std::collections::HashMap<String, i64>,
    clearance_fee_sat_vb: f64,
}

impl MempoolIndexer {
    pub fn new(clearance_fee_sat_vb: f64) -> Self {
        Self {
            clearance_fee_sat_vb,
            ..Default::default()
        }
    }

    pub fn ingest_verbose_map(
        &mut self,
        map: &std::collections::HashMap<String, VerboseMempoolEntry>,
        observed_at_ns: i64,
        node_height: u64,
        ibd_complete: bool,
        node_id: &str,
    ) -> Vec<MempoolTxEvent> {
        self.fee_rates.clear();
        let mut events = Vec::with_capacity(map.len().min(10_000));

        for (txid, entry) in map {
            let fee_rate = BtcRpcClient::fee_rate_sat_vb(entry);
            self.fee_rates.push(fee_rate);
            let first_seen = self
                .seen_txids
                .entry(txid.clone())
                .or_insert(observed_at_ns);
            events.push(MempoolTxEvent {
                schema_version: SCHEMA_VERSION.to_string(),
                observed_at_ns,
                txid: txid.clone(),
                first_seen_at_ns: *first_seen,
                fee_rate_sat_vb: fee_rate,
                vsize: entry.vsize,
                rbf_signaling: BtcRpcClient::rbf_signaling(entry),
                output_values_sats: vec![],
                output_addresses: vec![],
                mempool_entry: Some(BtcRpcClient::entry_meta(entry, entry.vsize)),
                node_sync_height: node_height,
                node_ibd_complete: ibd_complete,
                source_node_id: node_id.to_string(),
            });
        }
        events
    }

    pub fn fee_snapshot(
        &self,
        observed_at: &str,
        size_txs: u64,
        bytes: u64,
        min_fee_sat_vb: f64,
        node_id: &str,
    ) -> FeeSnapshot {
        let mut sorted = self.fee_rates.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p = |q: f64| -> f64 {
            if sorted.is_empty() {
                return 0.0;
            }
            let idx = (((sorted.len() as f64 - 1.0) * q).round() as usize).min(sorted.len() - 1);
            sorted[idx]
        };
        let stuck = if sorted.is_empty() {
            0.0
        } else {
            let stuck_n = sorted
                .iter()
                .filter(|f| **f < self.clearance_fee_sat_vb)
                .count();
            (stuck_n as f64 / sorted.len() as f64) * 100.0
        };

        FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: observed_at.to_string(),
            size_txs,
            bytes,
            min_fee_sat_vb,
            p50_fee_sat_vb: p(0.5),
            p90_fee_sat_vb: p(0.9),
            p99_fee_sat_vb: p(0.99),
            stuck_flow_pct: stuck,
            source_node_id: node_id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_snapshot_percentiles() {
        let mut idx = MempoolIndexer::new(10.0);
        idx.fee_rates = vec![1.0, 5.0, 10.0, 50.0, 100.0];
        let snap = idx.fee_snapshot("2026-05-20T00:00:00Z", 5, 1000, 1.0, "n1");
        assert!(snap.p50_fee_sat_vb > 0.0);
        assert!(snap.p99_fee_sat_vb >= snap.p50_fee_sat_vb);
    }
}
