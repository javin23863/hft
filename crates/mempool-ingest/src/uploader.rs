use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use reqwest::StatusCode;
use tokio::sync::mpsc;

use crate::metrics::IngestMetrics;

#[derive(Clone, Debug)]
pub struct UploadJob {
    pub local_path: PathBuf,
    pub object_key: String,
}

pub struct AsyncUploader {
    tx: mpsc::Sender<UploadJob>,
}

impl AsyncUploader {
    pub fn enqueue(&self, job: UploadJob) {
        let _ = self.tx.try_send(job);
    }
}

pub fn spawn_uploader(
    enable_upload: bool,
    endpoint: Option<String>,
    bucket: Option<String>,
    metrics: Arc<IngestMetrics>,
) -> Result<AsyncUploader> {
    let (tx, mut rx) = mpsc::channel::<UploadJob>(512);
    let endpoint = endpoint.unwrap_or_default();
    let bucket = bucket.unwrap_or_default();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    tokio::spawn(async move {
        while let Some(job) = rx.recv().await {
            metrics
                .upload_queue_depth
                .store(rx.len() as u64, Ordering::Relaxed);
            if !enable_upload {
                continue;
            }
            let body = match tokio::fs::read(&job.local_path).await {
                Ok(v) => v,
                Err(_) => {
                    metrics.upload_failures.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            };
            let mut ok = false;
            for attempt in 0..=4u64 {
                metrics.upload_attempts.fetch_add(1, Ordering::Relaxed);
                let url = format!(
                    "{}/{}/{}",
                    endpoint.trim_end_matches('/'),
                    bucket,
                    job.object_key.trim_start_matches('/')
                );
                match http.put(url).body(body.clone()).send().await {
                    Ok(resp)
                        if resp.status().is_success() || resp.status() == StatusCode::CREATED =>
                    {
                        ok = true;
                        break;
                    }
                    Ok(_) | Err(_) => {
                        tokio::time::sleep(Duration::from_millis(200 * (attempt + 1))).await;
                    }
                }
            }
            if ok {
                metrics.upload_success.fetch_add(1, Ordering::Relaxed);
            } else {
                metrics.upload_failures.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    Ok(AsyncUploader { tx })
}
