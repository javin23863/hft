# Gate 1 Runtime Stability

- Scope: Ingest polling stability, ZMQ SUB listener, partitioned bronze writes, and async B2 uploader.
- Validation command: `cargo test -p mempool-ingest`
- Pass condition (automated): ingest unit tests pass without panics; metrics counters exposed.
- Pass condition (operational): **24h continuous ingest** with stable poll latency, no spool growth runaway, and upload retry rate within SLO.

## Result

- Status: **PARTIAL** — automated tests PASS; 24h soak **PENDING** (not measured in CI).
- Notes: ZMQ uses `zmq` crate SUB thread; B2 upload uses `aws-sdk-s3` `put_object` with retries.
