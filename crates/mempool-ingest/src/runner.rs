use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use chrono::{Datelike, Timelike, Utc};
use mempool_core::IngestConfig;

use crate::bronze::BronzeWriter;
use crate::indexer::MempoolIndexer;
use crate::metrics::IngestMetrics;
use crate::node_gate::check_node_ready;
use crate::rpc::BtcRpcClient;
use crate::uploader::{spawn_uploader, AsyncUploader, UploadJob};
use crate::zmq_subscriber::{spawn_zmq_listener, SourceMode, ZmqEvent, ZmqSubscriber};

pub struct IngestRunner {
    cfg: IngestConfig,
    rpc: BtcRpcClient,
    indexer: MempoolIndexer,
    writer: BronzeWriter,
    metrics: Arc<IngestMetrics>,
    uploader: AsyncUploader,
    zmq: ZmqSubscriber,
    zmq_events: Option<tokio::sync::mpsc::UnboundedReceiver<ZmqEvent>>,
    source_mode: SourceMode,
}

impl IngestRunner {
    pub fn new(cfg: IngestConfig) -> Result<Self> {
        cfg.validate()?;
        let rpc = BtcRpcClient::new(&cfg)?;
        let indexer = MempoolIndexer::new(cfg.clearance_fee_sat_vb);
        let writer = BronzeWriter::new(&cfg.local_spool_dir, 2_000)?;
        let metrics = Arc::new(IngestMetrics::default());
        let uploader = spawn_uploader(
            cfg.enable_b2_upload,
            cfg.b2_endpoint.clone(),
            cfg.b2_bucket.clone(),
            cfg.b2_region.clone(),
            Arc::clone(&metrics),
        )?;
        let zmq = ZmqSubscriber::new(cfg.zmq_rawtx.clone(), cfg.zmq_hashblock.clone());
        let zmq_events = spawn_zmq_listener(cfg.zmq_rawtx.clone(), cfg.zmq_hashblock.clone());
        Ok(Self {
            cfg,
            rpc,
            indexer,
            writer,
            metrics,
            uploader,
            zmq,
            zmq_events,
            source_mode: SourceMode::Polling,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let health = check_node_ready(&self.rpc, self.cfg.max_tip_lag_blocks).await?;
        tracing::info!(
            node_height = health.height,
            ibd_complete = health.ibd_complete,
            "node ready; starting ingest loop"
        );
        self.source_mode = self.zmq.probe().await;
        if self.source_mode == SourceMode::Polling {
            tracing::warn!("ZMQ stream unavailable; using RPC polling mode");
        } else {
            tracing::info!("ZMQ source mode active");
        }

        let mut last_snapshot = std::time::Instant::now();
        let mut zmq_txids: HashSet<String> = HashSet::new();
        loop {
            if let Some(rx) = self.zmq_events.as_mut() {
                while let Ok(ev) = rx.try_recv() {
                    self.metrics.zmq_cycles.fetch_add(1, Ordering::Relaxed);
                    if let ZmqEvent::RawTx { txid } = ev {
                        zmq_txids.insert(txid);
                    }
                }
            }
            if let Err(e) = self.poll_once(&mut zmq_txids).await {
                self.metrics.rpc_errors.fetch_add(1, Ordering::Relaxed);
                tracing::error!(error = %e, "poll cycle failed");
            }
            self.metrics.poll_cycles.fetch_add(1, Ordering::Relaxed);
            if last_snapshot.elapsed() >= self.cfg.fee_snapshot_interval {
                self.metrics.log_summary();
                last_snapshot = std::time::Instant::now();
            }
            tokio::time::sleep(self.cfg.poll_interval).await;
        }
    }

    pub async fn poll_once(&mut self, zmq_txids: &mut HashSet<String>) -> Result<()> {
        let now = Utc::now();
        let run_date = format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day());
        let hour = format!("{:02}", now.hour());

        let chain = self.rpc.getblockchaininfo().await?;
        let info = self.rpc.getmempoolinfo().await?;
        let verbose = self.rpc.getrawmempool_verbose().await?;
        self.metrics
            .last_mempool_size
            .store(info.size, Ordering::Relaxed);

        let observed_at_ns = now.timestamp_nanos_opt().unwrap_or_else(|| now.timestamp() * 1_000_000_000);
        self.indexer.refresh_fee_rates(&verbose);
        let mut events = self.indexer.drain_new_events(
            &verbose,
            observed_at_ns,
            chain.blocks,
            !chain.initial_block_download,
            &self.cfg.node_id,
        );

        let mut enrich_ids: Vec<String> = zmq_txids.drain().collect();
        enrich_ids.truncate(self.cfg.max_zmq_enrich_per_poll);
        let mut addresses_by_txid = HashMap::new();
        for txid in enrich_ids {
            if let Ok(addrs) = self.rpc.getrawtransaction_verbose(&txid).await {
                addresses_by_txid.insert(txid, addrs);
            }
        }
        self.indexer
            .enrich_addresses(&mut events, &addresses_by_txid);

        if events.len() > self.cfg.max_new_tx_events_per_poll {
            events.truncate(self.cfg.max_new_tx_events_per_poll);
            self.metrics
                .tx_events_truncated
                .fetch_add(1, Ordering::Relaxed);
        }

        if let Some(chunk_path) = self.writer.write_events(&events, &run_date, &hour)? {
            self.metrics.bronze_chunks.fetch_add(1, Ordering::Relaxed);
            self.enqueue_upload(&chunk_path);
        }
        self.metrics
            .txs_indexed
            .fetch_add(events.len() as u64, Ordering::Relaxed);

        let min_fee_sat_vb = info.mempool_min_fee * 100_000.0;
        let snap = self.indexer.fee_snapshot(
            &now.to_rfc3339(),
            info.size,
            info.bytes,
            min_fee_sat_vb,
            &self.cfg.node_id,
        );
        let snap_path = self.writer.write_fee_snapshot_parquet(&snap, &run_date, &hour)?;
        self.enqueue_upload(&snap_path);
        Ok(())
    }

    pub async fn run_for(&mut self, cycles: usize) -> Result<()> {
        let _ = check_node_ready(&self.rpc, self.cfg.max_tip_lag_blocks).await?;
        let mut zmq_txids = HashSet::new();
        for _ in 0..cycles {
            self.poll_once(&mut zmq_txids).await?;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        self.writer.close()?;
        Ok(())
    }

    pub fn source_mode(&self) -> SourceMode {
        self.source_mode
    }

    fn enqueue_upload(&self, path: &std::path::Path) {
        if !self.cfg.enable_b2_upload {
            return;
        }
        let spool = PathBuf::from(&self.cfg.local_spool_dir);
        let object_key = path
            .strip_prefix(spool)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        self.uploader.enqueue(UploadJob {
            local_path: path.to_path_buf(),
            object_key,
        });
    }
}
