# Gate 3 Scenario Precision and Accuracy

## Scope

H1 precision, H2 FPR/accuracy, H3 correlation on **pipeline-exported** observations.

## Prerequisites (must complete first)

1. [LABEL_METHODOLOGY.md](../LABEL_METHODOLOGY.md)
2. >= 10 addresses in exchange registry
3. >= 50 rows in `data/feasibility/ground_truth/h1_labels.jsonl` (or plan threshold met)

## Commands

```bash
export HFT_RUN_DATE=YYYY-MM-DD
cargo run -p mempool-silver --bin mempool-silver-build
cargo run -p feasibility --bin feasibility-export
cargo run -p feasibility --bin feasibility-report
```

## Numeric PASS criteria

| Check | Threshold | Measured |
|-------|-----------|----------|
| H1 `n_obs` | >= 100 | 0 |
| H2 `n_obs` | >= 100 | 1751 |
| H3 `n_obs` | >= 60 | 200 |
| Registry addresses | >= 10 | 12 |
| H1 ground-truth rows | >= 50 | 50 |
| Enriched tx share | >= 10% | 0% (current run has no enriched H1 matches) |
| Report not placeholder | no "Placeholder" in FEASIBILITY_REPORT | PASS |

## Result

- Status: **PARTIAL**
- Result: H2/H3 sample thresholds pass; H1 remains below threshold due zero confirmed matches in this run.
- Commit:
- Run window: 2026-05-28 (bounded ingest, 200 cycles)
