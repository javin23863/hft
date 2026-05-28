# Evidence Index

This index links all gate artifacts required by the A-grade remediation checklist.

## Repro Commands

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p feasibility --bin feasibility-export`
- `cargo run -p feasibility --bin feasibility-report`
- `cargo run -p mempool-silver --bin mempool-silver-build`

## Gate Artifacts

- Gate 1 runtime stability: `docs/evidence/gate1_runtime_stability.md`
- Gate 1b feasibility decision: `docs/FEASIBILITY_REPORT.md`
- Gate 2 deterministic replay: `docs/evidence/gate2_replay_determinism.md`
- Gate 3 scenario precision/accuracy: `docs/evidence/gate3_scenario_accuracy.md`
- Gate 4 integration evidence: `docs/evidence/gate4_integration_tests.md`
- Gate 5 backtest and SOP thresholds: `docs/evidence/gate5_backtest_sop.md`
