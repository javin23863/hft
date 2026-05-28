use std::time::Duration;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceMode {
    Polling,
    Zmq,
}

pub struct ZmqSubscriber {
    rawtx: Option<String>,
    hashblock: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detects_zmq_mode_when_tcp_endpoint_is_reachable() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind listener");
        let addr = listener.local_addr().expect("local addr");
        let endpoint = format!("tcp://{addr}");
        let subscriber = ZmqSubscriber::new(Some(endpoint), None);

        let accept_task = tokio::spawn(async move {
            let _ = listener.accept().await;
        });
        let mode = subscriber.probe().await;
        accept_task.await.expect("join accept task");
        assert_eq!(mode, SourceMode::Zmq);
    }
}

impl ZmqSubscriber {
    pub fn new(rawtx: Option<String>, hashblock: Option<String>) -> Self {
        Self { rawtx, hashblock }
    }

    pub fn configured(&self) -> bool {
        self.rawtx.is_some() || self.hashblock.is_some()
    }

    pub async fn probe(&self) -> SourceMode {
        if !self.configured() {
            return SourceMode::Polling;
        }
        let mut ok = false;
        for endpoint in [self.rawtx.as_ref(), self.hashblock.as_ref()]
            .into_iter()
            .flatten()
        {
            if let Some(addr) = endpoint.strip_prefix("tcp://") {
                if tokio::time::timeout(
                    Duration::from_millis(300),
                    tokio::net::TcpStream::connect(addr),
                )
                .await
                .is_ok()
                {
                    ok = true;
                }
            }
        }
        if ok {
            SourceMode::Zmq
        } else {
            SourceMode::Polling
        }
    }

    pub async fn try_poll_event(&self) -> Result<bool> {
        Ok(matches!(self.probe().await, SourceMode::Zmq))
    }
}
