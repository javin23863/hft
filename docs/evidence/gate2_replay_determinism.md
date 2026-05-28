# Gate 2 Deterministic Replay Evidence

- Scope: typed Arrow/Parquet round-trip for fee snapshots, feature bars, and scenario signals.
- Validation command: `cargo test -p mempool-silver typed_parquet_roundtrip_validates_schema_and_hashes`
- Pass condition: repeated JSON digest for the same logical rows is identical; Parquet files expose named typed columns (not JSON blob columns).

## Result

- Status: **PASS** (validated by `contract_fidelity` test).
