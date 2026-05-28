# HFT Mempool Microstructure

Standalone Rust repository for Bitcoin mempool ingestion, feature extraction, feasibility testing, and scenario signal generation.

## Scope

- All code and artifacts live in this repository.
- No runtime wiring or code changes in external repositories.
- Phased flow:
  1. `mempool-ingest` writes bronze datasets.
  2. `mempool-silver` builds 1m features and scenario signal bars.
  3. `feasibility` computes KEEP/KILL decisions for H1/H2/H3.
  4. `scenario-engines` emits hypothesis-specific signals if feasibility passes.

## Crates

- `crates/mempool-core`: shared schema/config/types/lake-key helpers
- `crates/mempool-ingest`: RPC polling ingest + fee snapshots + bronze writer
- `crates/mempool-silver`: bronze-to-silver feature adapter and replay tools
- `crates/feasibility`: lag/correlation analysis + report generation
- `crates/scenario-engines`: H1/H2/H3 signal emission primitives

## Quick start

1. Copy `.env.example` to `.env` and fill credentials.
2. Run tests:

```bash
cargo test
```

3. Run ingest:

```bash
cargo run -p mempool-ingest --bin mempool-ingest
```

