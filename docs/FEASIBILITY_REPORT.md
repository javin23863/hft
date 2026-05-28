# Feasibility Report

## Outcomes
- H1: KILL
- H2: KILL
- H3: KEEP

## Metrics
- H1 lag median (sec): 2.9000 [1.8000, 4.2000] (n=5)
- H1 precision TP/(TP+FP): 0.7500 [0.2500, 1.0000] (n=5)
- H2 false-positive rate FP/(FP+TN): 0.3333 [0.0000, 1.0000] (n=5)
- H2 CPFP detection accuracy: 1.0000 [1.0000, 1.0000] (n=5)
- H3 correlation(congestion, fee_stress_proxy): 0.9954 [0.9863, 1.0000] (n=5)

Notes: Measured estimators with deterministic bootstrap CI. H1 rows require enriched outputs or explicit ground-truth labels. H2 actual uses stricter cpfp_consensus_truth than the operational detector. H3 fee_stress_proxy is a mempool z-score proxy (not external perp market data).
