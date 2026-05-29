# Gate 1 Runtime Stability

## Scope

24h continuous ingest, ZMQ + RPC, delta bronze writes, optional B2 upload timing.

## Validation

```bash
cargo test -p mempool-ingest
cargo run -p mempool-ingest -- --cycles 50   # requires Phase 0 CLI
# 24h:
RUST_LOG=info cargo run -p mempool-ingest 2>&1 | tee logs/soak/ingest-YYYYMMDD.log
```

## SLO pass thresholds

| SLO | Pass | Measured |
|-----|------|----------|
| Duration | >= 24h | |
| `rpc_errors / poll_cycles` | < 0.1% | |
| Spool growth (`du -sh runtime/spool`) | < 50 GB or documented | |
| `tx_events_truncated / txs_indexed` | < 1% | |
| `upload_failures / upload_attempts` | < 5% (if B2 on) | N/A |
| Poll p95 vs `HFT_POLL_MS` | < 2x configured interval | |

## B2 upload timing (A5)

| Run | Mean poll ms | p95 poll ms |
|-----|--------------|-------------|
| Upload off (`--cycles 200`) | | |
| Upload on (`--cycles 200`) | | |

**Pass:** upload-on p95 < 2x upload-off p95.

## Result

- Status: **PARTIAL**
- Result: bounded runtime ingest succeeds for 200 cycles with remote RPC+ZMQ tunnel; 24h soak still pending.
- Soak run started: `2026-05-28T06:25Z` (log: `logs/soak/ingest-20260528.log`)
- Log artifact: `logs/soak/ingest-YYYYMMDD.log` (gitignored)
