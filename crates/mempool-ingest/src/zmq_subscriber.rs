use std::thread;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
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
        let rawtx = self.rawtx.clone();
        let hashblock = self.hashblock.clone();
        let ok = tokio::task::spawn_blocking(move || probe_zmq_endpoints(rawtx, hashblock))
            .await
            .unwrap_or(false);
        if ok {
            SourceMode::Zmq
        } else {
            SourceMode::Polling
        }
    }
}

fn probe_zmq_lib_healthy() -> bool {
    let ctx = zmq::Context::new();
    let a = match ctx.socket(zmq::PAIR) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let b = match ctx.socket(zmq::PAIR) {
        Ok(s) => s,
        Err(_) => return false,
    };
    a.bind("inproc://hft-zmq-probe").is_ok() && b.connect("inproc://hft-zmq-probe").is_ok()
}

fn probe_zmq_endpoints(rawtx: Option<String>, hashblock: Option<String>) -> bool {
    if !probe_zmq_lib_healthy() {
        return false;
    }
    let ctx = zmq::Context::new();
    let socket = match ctx.socket(zmq::SUB) {
        Ok(s) => s,
        Err(_) => return false,
    };
    socket.set_rcvtimeo(1500).ok();
    let mut connected = false;
    if let Some(ep) = rawtx {
        if socket.connect(&ep).is_ok() {
            socket.set_subscribe(b"rawtx").ok();
            connected = true;
        }
    }
    if let Some(ep) = hashblock {
        if socket.connect(&ep).is_ok() {
            socket.set_subscribe(b"hashblock").ok();
            connected = true;
        }
    }
    connected
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
    let hash = Sha256::digest(raw);
    let mut rev = hash.to_vec();
    rev.reverse();
    Some(hex::encode(rev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn txid_from_raw_tx_produces_32_byte_hex() {
        let raw = vec![0x01, 0x00, 0x00, 0x00, 0x01];
        let txid = txid_from_raw_tx(&raw).expect("txid");
        assert_eq!(txid.len(), 64);
    }

    #[tokio::test]
    async fn probe_polling_when_not_configured() {
        let subscriber = ZmqSubscriber::new(None, None);
        assert_eq!(subscriber.probe().await, SourceMode::Polling);
    }
}
