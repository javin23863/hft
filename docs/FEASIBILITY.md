# Feasibility Framework (Phase 1b)

Hypotheses:

- H1: exchange inflow -> tradable lag
- H2: low-fee stuck flow -> settlement delay edge
- H3: mempool congestion -> perp stress

## KEEP/KILL criteria

- H1 KILL if median lag < 2s or label precision < 80%
- H2 KILL if stuck-flow false positive > 30% or CPFP/RBF accuracy < 90%
- H3 KILL if correlation < 0.15 or timestamps cannot align within 5 minutes

Output file:

- `docs/FEASIBILITY_REPORT.md`

