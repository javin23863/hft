use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use mempool_core::lake::bronze_tx_event_key;
use mempool_core::typed_parquet::write_fee_snapshots_typed;
use mempool_core::types::{FeeSnapshot, MempoolTxEvent};
use serde_json;

pub struct BronzeWriter {
    spool_dir: PathBuf,
    chunk_lines: usize,
    current_lines: usize,
    current_path: Option<PathBuf>,
    encoder: Option<GzEncoder<File>>,
}

impl BronzeWriter {
    pub fn new(spool_dir: impl AsRef<Path>, chunk_lines: usize) -> Result<Self> {
        let dir = spool_dir.as_ref();
        fs::create_dir_all(dir).context("create spool dir")?;
        Ok(Self {
            spool_dir: dir.to_path_buf(),
            chunk_lines: chunk_lines.max(100),
            current_lines: 0,
            current_path: None,
            encoder: None,
        })
    }

    fn rotate(&mut self, run_date: &str, hour: &str) -> Result<()> {
        if let Some(enc) = self.encoder.take() {
            let mut f = enc.finish().context("gzip finish")?;
            f.flush().ok();
        }
        self.current_lines = 0;
        let chunk_ms = chrono::Utc::now().timestamp_millis() as u64;
        let key = bronze_tx_event_key(run_date, hour, chunk_ms);
        let path = self.spool_dir.join(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let file = File::create(&path).context("create chunk")?;
        self.encoder = Some(GzEncoder::new(file, Compression::fast()));
        self.current_path = Some(path);
        Ok(())
    }

    pub fn write_events(
        &mut self,
        events: &[MempoolTxEvent],
        run_date: &str,
        hour: &str,
    ) -> Result<Option<PathBuf>> {
        if events.is_empty() {
            return Ok(None);
        }
        if self.encoder.is_none() {
            self.rotate(run_date, hour)?;
        }
        let enc = self.encoder.as_mut().context("encoder")?;
        for ev in events {
            let line = serde_json::to_string(ev).context("serialize event")?;
            writeln!(enc, "{line}").context("write line")?;
            self.current_lines += 1;
            if self.current_lines >= self.chunk_lines {
                let path = self.current_path.clone();
                self.rotate(run_date, hour)?;
                return Ok(path);
            }
        }
        Ok(self.current_path.clone())
    }

    pub fn write_fee_snapshot_parquet(
        &self,
        snap: &FeeSnapshot,
        run_date: &str,
        hour: &str,
    ) -> Result<PathBuf> {
        let ts_ms = chrono::Utc::now().timestamp_millis();
        let path = self.spool_dir.join(format!(
            "hft/bronze/source=bitcoin/dataset=mempool_fee_snapshot/run={run_date}/hour={hour}/part-{ts_ms}.parquet"
        ));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        write_fee_snapshots_typed(&path, std::slice::from_ref(snap))
            .context("write typed fee snapshot parquet")?;
        Ok(path)
    }

    pub fn close(&mut self) -> Result<()> {
        if let Some(enc) = self.encoder.take() {
            let mut f = enc.finish().context("gzip finish")?;
            f.flush().ok();
        }
        Ok(())
    }
}
