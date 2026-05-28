# HFT Mempool Microstructure

Standalone Rust repository for **Bitcoin mempool observability**: bronze ingest, silver features, feasibility gates (H1/H2/H3), and scenario signals.

This is **not** a live MEV or latency-arbitrage execution stack. There is no bundle detection, propagation race model, or trading PnL path here.

## Scope

- All code and artifacts live in this repository.
- No runtime wiring or code changes in external repositories.
- Bronze ingest writes **delta** tx events (new mempool txids only) plus periodic fee snapshots.
- Exchange inflow labels: `data/exchange_registry/v1/exchanges.json` + `labeled_deposits.json`, optional `data/feasibility/ground_truth/h1_labels.jsonl`.
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
cargo clippy --workspace --all-targets -- -D warnings
```

3. Export feasibility from spool (after ingest) and render report:

```bash
cargo run -p feasibility --bin feasibility-export
cargo run -p feasibility --bin feasibility-report
```

4. Run ingest:

```bash
cargo run -p mempool-ingest --bin mempool-ingest
```

