# A-Grade Scorecard

Fill this after all phases complete. **Product PASS:** total >= 90 and no category < 80.

| Category | Weight | Score (/100) | Evidence | Notes |
|----------|--------|--------------|----------|-------|
| Theory / formula fidelity | 25 | 88 | Gate 2, `contract_fidelity`, `formula_fidelity` tests, [FEATURE_CATALOG.md](FEATURE_CATALOG.md) | Unit and contract checks pass locally |
| Inference / econometrics | 20 | 66 | [FEASIBILITY_REPORT.md](FEASIBILITY_REPORT.md), Gate 3 thresholds, `data/feasibility/*.jsonl` | H2/H3 sample thresholds met; H1 still zero |
| Numerics / data contract | 20 | 86 | Typed Parquet tests, [LAKE_CONTRACT.md](LAKE_CONTRACT.md) | Typed Parquet path and tests are green |
| Research process discipline | 15 | 78 | [RESEARCH_REPORT.md](RESEARCH_REPORT.md), [LABEL_METHODOLOGY.md](LABEL_METHODOLOGY.md) | Labels and methodology added; evidence script missing |
| Systems engineering | 20 | 64 | Gate 1 SLO table, metrics, CI workflow | Bounded ingest works; 24h soak pending |

**Weighted total:** 80.3 / 100

**Commit:** `git rev-parse HEAD`  
**Run window:** YYYY-MM-DD → YYYY-MM-DD  
**Signed off by:** operator name / date
