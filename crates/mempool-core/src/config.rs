use std::env;
use std::time::Duration;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub btc_rpc_url: String,
    pub btc_rpc_user: String,
    pub btc_rpc_pass: String,
    pub zmq_rawtx: Option<String>,
    pub zmq_hashblock: Option<String>,
    pub node_id: String,
    pub local_spool_dir: String,
    pub fee_snapshot_interval: Duration,
    pub poll_interval: Duration,
    pub clearance_fee_sat_vb: f64,
    pub max_tip_lag_blocks: u64,
    pub b2_bucket: Option<String>,
    pub b2_endpoint: Option<String>,
    pub b2_region: Option<String>,
    pub enable_b2_upload: bool,
    pub max_new_tx_events_per_poll: usize,
    pub max_zmq_enrich_per_poll: usize,
}

impl IngestConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            btc_rpc_url: env::var("BITCOIN_RPC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:18332/".into()),
            btc_rpc_user: env::var("BITCOIN_RPC_USER").unwrap_or_else(|_| "rpcuser".into()),
            btc_rpc_pass: env::var("BITCOIN_RPC_PASS").unwrap_or_default(),
            zmq_rawtx: env::var("BITCOIN_ZMQ_RAWTX").ok(),
            zmq_hashblock: env::var("BITCOIN_ZMQ_HASHBLOCK").ok(),
            node_id: env::var("HFT_NODE_ID").unwrap_or_else(|_| "hft-local-1".into()),
            local_spool_dir: env::var("HFT_LOCAL_SPOOL_DIR")
                .unwrap_or_else(|_| "/tmp/hft-spool".into()),
            fee_snapshot_interval: Duration::from_secs(
                env::var("HFT_FEE_SNAPSHOT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(15),
            ),
            poll_interval: Duration::from_millis(
                env::var("HFT_POLL_MS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(250),
            ),
            clearance_fee_sat_vb: env::var("HFT_CLEARANCE_FEE_SAT_VB")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10.0),
            max_tip_lag_blocks: env::var("HFT_MAX_TIP_LAG_BLOCKS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            b2_bucket: env::var("B2_BUCKET").ok(),
            b2_endpoint: env::var("B2_ENDPOINT_URL")
                .or_else(|_| env::var("AWS_ENDPOINT_URL"))
                .ok(),
            b2_region: env::var("B2_REGION")
                .or_else(|_| env::var("AWS_REGION"))
                .ok(),
            enable_b2_upload: env::var("HFT_ENABLE_B2_UPLOAD")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            max_new_tx_events_per_poll: env::var("HFT_MAX_NEW_TX_EVENTS_PER_POLL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2_000),
            max_zmq_enrich_per_poll: env::var("HFT_MAX_ZMQ_ENRICH_PER_POLL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        })
    }

    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            !self.btc_rpc_url.is_empty(),
            "BITCOIN_RPC_URL required"
        );
        if self.enable_b2_upload {
            self.b2_bucket
                .as_ref()
                .context("B2_BUCKET required when HFT_ENABLE_B2_UPLOAD=1")?;
            self.b2_endpoint
                .as_ref()
                .context("B2_ENDPOINT_URL required when HFT_ENABLE_B2_UPLOAD=1")?;
        }
        Ok(())
    }
}
