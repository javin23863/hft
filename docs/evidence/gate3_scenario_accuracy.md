# Gate 3 Scenario Precision and Accuracy

- Scope: H1 precision, H2 FPR and CPFP detection accuracy, H3 correlation.
- Source: `data/feasibility/*.jsonl`
- Validation command: `cargo run -p feasibility --bin feasibility-report`

## Result

- Status: PASS when `docs/FEASIBILITY_REPORT.md` shows explicit `n_obs`, CI bounds, and KEEP/KILL outcomes.
