use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use mempool_core::config::IngestConfig;
use mempool_core::types::MempoolEntryMeta;

#[derive(Debug)]
pub struct BtcRpcClient {
    http: Client,
    url: String,
    auth_header: String,
    req_id: AtomicU64,
}

#[derive(Debug, Deserialize)]
pub struct ChainInfo {
    pub blocks: u64,
    pub headers: u64,
    #[serde(rename = "initialblockdownload")]
    pub initial_block_download: bool,
}

#[derive(Debug, Deserialize)]
pub struct MempoolInfo {
    pub size: u64,
    pub bytes: u64,
    #[serde(rename = "mempoolminfee")]
    pub mempool_min_fee: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerboseMempoolEntry {
    pub fee: f64,
    pub vsize: u32,
    pub time: u64,
    #[serde(default)]
    pub descendantcount: u32,
    #[serde(default)]
    pub ancestorcount: u32,
    #[serde(default)]
    pub ancestorsize: u32,
    #[serde(default)]
    pub descendantssize: u32,
    #[serde(rename = "bip125-replaceable")]
    pub bip125_replaceable: Option<bool>,
}

impl BtcRpcClient {
    pub fn new(cfg: &IngestConfig) -> Result<Self> {
        use base64::Engine;
        let token = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", cfg.btc_rpc_user, cfg.btc_rpc_pass));
        Ok(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .context("http client")?,
            url: cfg.btc_rpc_url.trim_end_matches('/').to_string(),
            auth_header: format!("Basic {token}"),
            req_id: AtomicU64::new(1),
        })
    }

    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.req_id.fetch_add(1, Ordering::Relaxed);
        let body = json!({
            "jsonrpc": "1.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let resp = self
            .http
            .post(&self.url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "text/plain")
            .body(body.to_string())
            .send()
            .await
            .context("rpc post")?;
        let status = resp.status();
        let text = resp.text().await.context("rpc body")?;
        if !status.is_success() {
            anyhow::bail!("rpc http {status}: {text}");
        }
        let v: Value = serde_json::from_str(&text).context("rpc json")?;
        if let Some(err) = v.get("error") {
            if !err.is_null() {
                anyhow::bail!("rpc error: {err}");
            }
        }
        v.get("result").cloned().context("rpc missing result")
    }

    pub async fn getblockchaininfo(&self) -> Result<ChainInfo> {
        let r = self.call("getblockchaininfo", json!([])).await?;
        serde_json::from_value(r).context("parse chain info")
    }

    pub async fn getmempoolinfo(&self) -> Result<MempoolInfo> {
        let r = self.call("getmempoolinfo", json!([])).await?;
        serde_json::from_value(r).context("parse mempool info")
    }

    pub async fn getrawmempool_verbose(
        &self,
    ) -> Result<std::collections::HashMap<String, VerboseMempoolEntry>> {
        let r = self.call("getrawmempool", json!([true])).await?;
        serde_json::from_value(r).context("parse verbose mempool")
    }

    pub async fn getmempoolentry(&self, txid: &str) -> Result<VerboseMempoolEntry> {
        let r = self.call("getmempoolentry", json!([txid])).await?;
        serde_json::from_value(r).context("parse mempool entry")
    }

    pub fn entry_meta(entry: &VerboseMempoolEntry, vsize: u32) -> MempoolEntryMeta {
        let fee_sat = (entry.fee * 100_000_000.0).round() as u64;
        let fee_rate = if vsize > 0 {
            fee_sat as f64 / vsize as f64
        } else {
            0.0
        };
        MempoolEntryMeta {
            ancestor_count: entry.ancestorcount,
            descendant_count: entry.descendantcount,
            ancestor_feerate_sat_vb: Some(fee_rate),
            descendant_feerate_sat_vb: None,
        }
    }

    pub fn fee_rate_sat_vb(entry: &VerboseMempoolEntry) -> f64 {
        let fee_sat = (entry.fee * 100_000_000.0).round() as u64;
        if entry.vsize > 0 {
            fee_sat as f64 / entry.vsize as f64
        } else {
            0.0
        }
    }

    pub fn rbf_signaling(entry: &VerboseMempoolEntry) -> bool {
        entry.bip125_replaceable.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn rpc_parses_mempoolinfo() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": {
                    "size": 100,
                    "bytes": 200000,
                    "mempoolminfee": 0.00001
                },
                "error": null,
                "id": 1
            })))
            .mount(&server)
            .await;

        let cfg = IngestConfig {
            btc_rpc_url: server.uri(),
            btc_rpc_user: "u".into(),
            btc_rpc_pass: "p".into(),
            zmq_rawtx: None,
            zmq_hashblock: None,
            node_id: "test".into(),
            local_spool_dir: "/tmp/hft-test".into(),
            fee_snapshot_interval: std::time::Duration::from_secs(15),
            poll_interval: std::time::Duration::from_millis(250),
            clearance_fee_sat_vb: 10.0,
            max_tip_lag_blocks: 2,
            b2_bucket: None,
            b2_endpoint: None,
            enable_b2_upload: false,
        };
        let client = BtcRpcClient::new(&cfg).unwrap();
        let info = client.getmempoolinfo().await.unwrap();
        assert_eq!(info.size, 100);
    }
}
