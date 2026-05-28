# Bitcoin Node Operations

## Modes

| Mode | Description |
|------|-------------|
| Shared tunnel (default) | SSH tunnel to your Bitcoin full node, RPC local URL like `http://127.0.0.1:18332/` |
| Dedicated node | Local bitcoind on ingest host |

## Startup checks

Ingest refuses to start unless:

1. RPC reachable
2. `initialblockdownload=false`
3. tip lag within `HFT_MAX_TIP_LAG_BLOCKS` (default 2)
4. Polling loop healthy (ZMQ optional fallback)

## Minimum Bitcoin Core

Pin to Bitcoin Core version that supports:

- `getrawmempool true`
- `getmempoolentry`
- `getmempoolinfo`

Recommended: v27.1+.

## Security

- Keep RPC credentials out of git.
- Keep `.env` and tunnel host secrets local only.

