# Lake Contract (hft-owned namespace)

All datasets are emitted under `hft/`.

## Bronze

- `hft/bronze/source=bitcoin/dataset=mempool_tx_event/run=YYYY-MM-DD/HH/chunk_<ms>.jsonl.gz`
- `hft/bronze/source=bitcoin/dataset=mempool_fee_snapshot/run=YYYY-MM-DD/snapshot.jsonl`

## Silver

- `hft/silver/source=bitcoin/dataset=mempool_features_1m/run=YYYY-MM-DD/features.parquet`
- `hft/silver/source=bitcoin/dataset=scenario_signal/run=YYYY-MM-DD/signals.parquet`

Note: current implementation writes JSONL content; file suffixes are reserved for future parquet migration.

## Versioning

- `schema_version` is mandatory in every record.
- Current version: `1.0.0`.

