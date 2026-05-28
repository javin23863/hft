use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct IngestMetrics {
    pub poll_cycles: AtomicU64,
    pub rpc_errors: AtomicU64,
    pub txs_indexed: AtomicU64,
    pub bronze_chunks: AtomicU64,
    pub last_mempool_size: AtomicU64,
    pub zmq_cycles: AtomicU64,
    pub upload_attempts: AtomicU64,
    pub upload_success: AtomicU64,
    pub upload_failures: AtomicU64,
    pub upload_queue_depth: AtomicU64,
}

impl IngestMetrics {
    pub fn log_summary(&self) {
        tracing::info!(
            poll_cycles = self.poll_cycles.load(Ordering::Relaxed),
            rpc_errors = self.rpc_errors.load(Ordering::Relaxed),
            txs_indexed = self.txs_indexed.load(Ordering::Relaxed),
            bronze_chunks = self.bronze_chunks.load(Ordering::Relaxed),
            mempool_size = self.last_mempool_size.load(Ordering::Relaxed),
            zmq_cycles = self.zmq_cycles.load(Ordering::Relaxed),
            upload_attempts = self.upload_attempts.load(Ordering::Relaxed),
            upload_success = self.upload_success.load(Ordering::Relaxed),
            upload_failures = self.upload_failures.load(Ordering::Relaxed),
            upload_queue_depth = self.upload_queue_depth.load(Ordering::Relaxed),
            "ingest metrics"
        );
    }
}
