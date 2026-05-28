use anyhow::{bail, Result};

use crate::rpc::BtcRpcClient;

pub struct NodeHealth {
    pub height: u64,
    pub ibd_complete: bool,
}

pub async fn check_node_ready(rpc: &BtcRpcClient, max_tip_lag_blocks: u64) -> Result<NodeHealth> {
    let chain = rpc.getblockchaininfo().await?;
    if chain.initial_block_download {
        bail!("node still in initial block download — ingest refused");
    }
    let lag = chain.headers.saturating_sub(chain.blocks);
    if lag > max_tip_lag_blocks {
        bail!("node tip lag {lag} blocks exceeds max {max_tip_lag_blocks}");
    }
    Ok(NodeHealth {
        height: chain.blocks,
        ibd_complete: !chain.initial_block_download,
    })
}
