# Feasibility Report

Status: Measured from local pipeline artifacts (`run=2026-05-28`) after successful bounded ingest.

## Hypothesis outcomes

- H1: KILL (no confirmed inflow matches in current sample)
- H2: KILL (insufficient positive precision under current thresholding)
- H3: KEEP (sample threshold met; proxy correlation available)

## Observations

- H1 observations: 0
- H2 observations: 1751
- H3 observations: 200
- Exchange registry addresses: 12
- H1 ground-truth labels: 50

## Notes

- Runtime compatibility fix: ingest now supports remote node verbose mempool fee formats where `fee` may be absent or object-shaped.
- Reproduce this report:
  - `HFT_RUN_DATE=2026-05-28 cargo run -p mempool-silver --bin mempool-silver-build`
  - `HFT_RUN_DATE=2026-05-28 cargo run -p feasibility --bin feasibility-export`
  - `HFT_RUN_DATE=2026-05-28 cargo run -p feasibility --bin feasibility-report`

