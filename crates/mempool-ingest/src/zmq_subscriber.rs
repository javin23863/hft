use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceMode {
    Polling,
    Zmq,
}

#[derive(Debug, Clone)]
pub enum ZmqEvent {
    RawTx { txid: String },
    HashBlock { hash: String },
}

pub struct ZmqSubscriber {
    rawtx: Option<String>,
    hashblock: Option<String>,
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
        for endpoint in [self.rawtx.as_ref(), self.hashblock.as_ref()].into_iter().flatten() {
            if let Some(addr) = endpoint.strip_prefix("tcp://") {
                if tokio::time::timeout(Duration::from_millis(300), tokio::net::TcpStream::connect(addr))
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
}

pub fn spawn_zmq_listener(
    rawtx: Option<String>,
    hashblock: Option<String>,
) -> Option<mpsc::UnboundedReceiver<ZmqEvent>> {
    if rawtx.is_none() && hashblock.is_none() {
        return None;
    }
    let (tx, rx) = mpsc::unbounded_channel();
    thread::spawn(move || {
        if let Err(e) = zmq_listener_loop(rawtx, hashblock, tx) {
            warn!(error = %e, "zmq listener exited");
        }
    });
    Some(rx)
}

fn zmq_listener_loop(
    rawtx: Option<String>,
    hashblock: Option<String>,
    tx: mpsc::UnboundedSender<ZmqEvent>,
) -> Result<()> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SUB).context("create zmq SUB")?;
    socket.set_rcvtimeo(500).ok();
    if let Some(ep) = &rawtx {
        socket.connect(ep).context("connect rawtx zmq")?;
        socket.set_subscribe(b"rawtx").context("subscribe rawtx")?;
    }
    if let Some(ep) = &hashblock {
        socket.connect(ep).context("connect hashblock zmq")?;
        socket.set_subscribe(b"hashblock").context("subscribe hashblock")?;
    }

    loop {
        let msg = match socket.recv_multipart(0) {
            Ok(m) => m,
            Err(zmq::Error::EAGAIN) => continue,
            Err(e) => return Err(e.into()),
        };
        if msg.is_empty() {
            continue;
        }
        let topic = msg[0].as_slice();
        if topic == b"rawtx" && msg.len() >= 2 {
            if let Some(txid) = txid_from_raw_tx(&msg[1]) {
                let _ = tx.send(ZmqEvent::RawTx { txid });
            }
        } else if topic == b"hashblock" && msg.len() >= 2 {
            let hash = hex::encode(&msg[1]);
            let _ = tx.send(ZmqEvent::HashBlock { hash });
        }
    }
}

fn txid_from_raw_tx(raw: &[u8]) -> Option<String> {
    if raw.len() < 4 {
        return None;
    }
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(raw);
    let hash = hasher.finalize();
    let mut rev = hash.to_vec();
    rev.reverse();
    Some(hex::encode(&rev))
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
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
