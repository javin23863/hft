use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;
use aws_sdk_s3::primitives::ByteStream;
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
        if self.tx.try_send(job).is_err() {
            tracing::warn!("upload queue full; dropping job");
        }
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

    tokio::spawn(async move {
        let client = if enable_upload {
            let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
            if !endpoint.is_empty() {
                loader = loader.endpoint_url(endpoint);
            }
            let cfg = loader.load().await;
            Some(aws_sdk_s3::Client::new(&cfg))
        } else {
            None
        };

        while let Some(job) = rx.recv().await {
            metrics
                .upload_queue_depth
                .store(rx.len() as u64, Ordering::Relaxed);
            if !enable_upload {
                continue;
            }
            let Some(client) = client.as_ref() else {
                continue;
            };
            let body = match tokio::fs::read(&job.local_path).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(error = %e, path = ?job.local_path, "read upload file failed");
                    metrics.upload_failures.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            };
            let mut ok = false;
            for attempt in 0..=4u64 {
                metrics.upload_attempts.fetch_add(1, Ordering::Relaxed);
                let send = client
                    .put_object()
                    .bucket(&bucket)
                    .key(job.object_key.trim_start_matches('/'))
                    .body(ByteStream::from(body.clone()))
                    .send()
                    .await;
                match send {
                    Ok(_) => {
                        ok = true;
                        break;
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, attempt, "s3 put_object retry");
                        tokio::time::sleep(std::time::Duration::from_millis(200 * (attempt + 1)))
                            .await;
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
