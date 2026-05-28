use std::collections::{HashMap, HashSet};

use mempool_core::types::{FeeSnapshot, MempoolTxEvent};
use mempool_core::schema::SCHEMA_VERSION;

use crate::rpc::{BtcRpcClient, VerboseMempoolEntry};

#[derive(Debug, Default)]
pub struct MempoolIndexer {
    fee_rates: Vec<f64>,
    seen_txids: HashMap<String, i64>,
    active_mempool: HashSet<String>,
    clearance_fee_sat_vb: f64,
}

impl MempoolIndexer {
    pub fn new(clearance_fee_sat_vb: f64) -> Self {
        Self {
            clearance_fee_sat_vb,
            ..Default::default()
        }
    }

    pub fn enrich_addresses(
        &self,
        events: &mut [MempoolTxEvent],
        addresses_by_txid: &HashMap<String, Vec<Option<String>>>,
    ) {
        for ev in events.iter_mut() {
            if let Some(addrs) = addresses_by_txid.get(&ev.txid) {
                ev.output_addresses = addrs.clone();
            }
        }
    }

    /// Refresh aggregate fee-rate distribution from the full mempool map (one RPC view).
    pub fn refresh_fee_rates(&mut self, map: &HashMap<String, VerboseMempoolEntry>) {
        self.fee_rates.clear();
        self.active_mempool.clear();
        for (txid, entry) in map {
            self.active_mempool.insert(txid.clone());
            self.fee_rates.push(BtcRpcClient::fee_rate_sat_vb(entry));
        }
    }

    /// Emit bronze events only for txids newly seen in the mempool (delta).
    pub fn drain_new_events(
        &mut self,
        map: &HashMap<String, VerboseMempoolEntry>,
        observed_at_ns: i64,
        node_height: u64,
        ibd_complete: bool,
        node_id: &str,
    ) -> Vec<MempoolTxEvent> {
        let mut events = Vec::new();
        for (txid, entry) in map {
            if !self.active_mempool.contains(txid) {
                continue;
            }
            if self.seen_txids.contains_key(txid) {
                continue;
            }
            let fee_rate = BtcRpcClient::fee_rate_sat_vb(entry);
            self.seen_txids.insert(txid.clone(), observed_at_ns);
            events.push(MempoolTxEvent {
                schema_version: SCHEMA_VERSION.to_string(),
                observed_at_ns,
                txid: txid.clone(),
                first_seen_at_ns: observed_at_ns,
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
        self.prune_seen_txids();
        events
    }

    fn prune_seen_txids(&mut self) {
        self.seen_txids
            .retain(|txid, _| self.active_mempool.contains(txid));
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
    use crate::rpc::VerboseMempoolEntry;

    fn entry(fee_btc: f64, vsize: u32) -> VerboseMempoolEntry {
        VerboseMempoolEntry {
            fee: fee_btc,
            vsize,
            time: 0,
            fees: None,
            descendantcount: 1,
            ancestorcount: 1,
            ancestorsize: vsize,
            descendantssize: vsize,
            bip125_replaceable: Some(false),
        }
    }

    #[test]
    fn fee_snapshot_percentiles() {
        let mut idx = MempoolIndexer::new(10.0);
        idx.fee_rates = vec![1.0, 5.0, 10.0, 50.0, 100.0];
        let snap = idx.fee_snapshot("2026-05-20T00:00:00Z", 5, 1000, 1.0, "n1");
        assert!(snap.p50_fee_sat_vb > 0.0);
        assert!(snap.p99_fee_sat_vb >= snap.p50_fee_sat_vb);
    }

    #[test]
    fn drain_new_events_only_emits_once_per_txid() {
        let mut idx = MempoolIndexer::new(10.0);
        let mut map = HashMap::new();
        map.insert("txa".into(), entry(5000.0, 200));
        idx.refresh_fee_rates(&map);
        let first = idx.drain_new_events(&map, 100, 1, true, "n1");
        assert_eq!(first.len(), 1);
        let second = idx.drain_new_events(&map, 200, 1, true, "n1");
        assert!(second.is_empty());
    }
}
