# Gate 3 Scenario Precision and Accuracy

- Scope: H1 precision, H2 FPR and CPFP detection accuracy, H3 correlation.
- Source: `data/feasibility/*.jsonl` (prefer `feasibility-export` from bronze/silver replay).
- Validation commands:
  - `cargo run -p feasibility --bin feasibility-export` (pipeline-derived observations)
  - `cargo run -p feasibility --bin feasibility-report`

## Result

- Status: **PARTIAL** — report generation and estimators PASS on available `n`; live-labeled ground truth at scale **PENDING**.
- Notes: populate `data/exchange_registry/v1/exchanges.json` addresses for H1 inflow labels.
