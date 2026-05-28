# Gate 2 Deterministic Replay Evidence

- Scope: parquet round-trip and deterministic hash verification.
- Validation command: `cargo test -p mempool-silver parquet_roundtrip_validates_schema_and_hashes`
- Pass condition: repeated hash digest for the same replayed rows is identical.

## Result

- Status: PASS (validated by `contract_fidelity` test).
