# H1 / H2 Label Methodology

## Purpose

Ground-truth labels for feasibility export must be **independent** of the operational detectors documented in `crates/mempool-core/src/features.rs`.

## H1 — Exchange inflow

| Field | Definition |
|-------|------------|
| **Predicted** | `predict_exchange_inflow` — fee rate and vsize heuristic (no address registry) |
| **Actual (default)** | `confirm_exchange_inflow` — enriched output address matches [exchange registry](../data/exchange_registry/v1/) |
| **Actual (override)** | Row in `data/feasibility/ground_truth/h1_labels.jsonl`: `{"txid":"…","actual_positive":true\|false}` |

### Sampling frame

- Time window: same `run=YYYY-MM-DD` as bronze spool / 24h soak
- Include both positive and negative labels where possible
- Target: **>= 50** labeled txids or **>= 5%** of enriched events (plan threshold)

### Sources (document per address)

- Public label datasets (WalletExplorer tags, exchange disclosure, court/regulatory filings)
- Manual audit of block explorer cluster pages
- Record `source` + `as_of` in registry `sources` or per-row comment file

## H2 — CPFP

| Field | Definition |
|-------|------------|
| **Predicted** | `detect_cpfp` — operational thresholds |
| **Actual (default)** | `cpfp_consensus_truth` — stricter descendant count and feerate gaps |
| **Actual (override)** | Optional `data/feasibility/ground_truth/h2_labels.jsonl` |

## H3 — Fee stress proxy

- **Not** external perp market data
- `fee_stress_proxy_score` derived from mempool z-score and stuck flow
- KEEP on H3 supports **telemetry correlation only**, not tradable perp edge

## Reproducibility

```bash
export HFT_RUN_DATE=YYYY-MM-DD
cargo run -p feasibility --bin feasibility-export
cargo run -p feasibility --bin feasibility-report
```
