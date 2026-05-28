use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolEntryMeta {
    pub ancestor_count: u32,
    pub descendant_count: u32,
    pub ancestor_feerate_sat_vb: Option<f64>,
    pub descendant_feerate_sat_vb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolTxEvent {
    pub schema_version: String,
    pub observed_at_ns: i64,
    pub txid: String,
    pub first_seen_at_ns: i64,
    pub fee_rate_sat_vb: f64,
    pub vsize: u32,
    pub rbf_signaling: bool,
    pub output_values_sats: Vec<u64>,
    pub output_addresses: Vec<Option<String>>,
    pub mempool_entry: Option<MempoolEntryMeta>,
    pub node_sync_height: u64,
    pub node_ibd_complete: bool,
    pub source_node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeSnapshot {
    pub schema_version: String,
    pub observed_at: String,
    pub size_txs: u64,
    pub bytes: u64,
    pub min_fee_sat_vb: f64,
    pub p50_fee_sat_vb: f64,
    pub p90_fee_sat_vb: f64,
    pub p99_fee_sat_vb: f64,
    pub stuck_flow_pct: f64,
    pub source_node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPosteriorFields {
    pub mean: f64,
    pub std: f64,
    pub n_obs: u64,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolFeatureBar {
    pub schema_version: String,
    pub ts_minute: String,
    pub tx_count: u64,
    pub avg_fee_sat_vb: f64,
    pub p90_fee_sat_vb: f64,
    pub p99_fee_sat_vb: f64,
    pub stuck_flow_pct: f64,
    pub fee_spike_zscore: f64,
    pub exchange_inflow_event: bool,
    pub cpfp_detected: bool,
    pub congestion_regime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSignal {
    pub schema_version: String,
    pub observed_at: String,
    pub hypothesis_id: String,
    pub signal_name: String,
    pub confidence: f64,
    pub posterior: SignalPosteriorFields,
    pub payload: serde_json::Value,
}
