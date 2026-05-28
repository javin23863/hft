use mempool_core::{FeeSnapshot, MempoolFeatureBar, SCHEMA_VERSION};
use mempool_core::ExchangeRegistry;
use mempool_silver::{build_feature_bars, run_regime_backtest};
use std::collections::HashSet;

#[test]
fn zscore_is_zero_when_variance_is_zero() {
    let snaps = vec![
        FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: "2026-05-20T00:00:00Z".into(),
            size_txs: 10,
            bytes: 10,
            min_fee_sat_vb: 1.0,
            p50_fee_sat_vb: 5.0,
            p90_fee_sat_vb: 8.0,
            p99_fee_sat_vb: 10.0,
            stuck_flow_pct: 15.0,
            source_node_id: "n1".into(),
        },
        FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: "2026-05-20T00:01:00Z".into(),
            size_txs: 11,
            bytes: 11,
            min_fee_sat_vb: 1.1,
            p50_fee_sat_vb: 5.1,
            p90_fee_sat_vb: 8.1,
            p99_fee_sat_vb: 10.0,
            stuck_flow_pct: 16.0,
            source_node_id: "n1".into(),
        },
    ];
    let reg = ExchangeRegistry::from_addresses(HashSet::new());
    let bars = build_feature_bars(&snaps, &[], &reg, 10.0);
    assert_eq!(bars[0].fee_spike_zscore, 0.0);
    assert_eq!(bars[1].fee_spike_zscore, 0.0);
}

#[test]
fn congestion_regime_boundaries_are_stable() {
    let mk = |stuck: f64| FeeSnapshot {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at: "2026-05-20T00:00:00Z".into(),
        size_txs: 10,
        bytes: 10,
        min_fee_sat_vb: 1.0,
        p50_fee_sat_vb: 2.0,
        p90_fee_sat_vb: 3.0,
        p99_fee_sat_vb: 4.0,
        stuck_flow_pct: stuck,
        source_node_id: "n1".into(),
    };
    let reg = ExchangeRegistry::from_addresses(HashSet::new());
    let bars = build_feature_bars(&[mk(20.0), mk(21.0), mk(70.0), mk(69.0)], &[], &reg, 10.0);
    assert_eq!(bars[0].congestion_regime, "clearing");
    assert_eq!(bars[1].congestion_regime, "normal");
    assert_eq!(bars[2].congestion_regime, "congested");
    assert_eq!(bars[3].congestion_regime, "normal");
}

#[test]
fn backtest_delta_matches_mean_difference_identity() {
    let features = vec![
        MempoolFeatureBar {
            schema_version: SCHEMA_VERSION.to_string(),
            ts_minute: "t1".into(),
            tx_count: 1,
            avg_fee_sat_vb: 1.0,
            p90_fee_sat_vb: 1.0,
            p99_fee_sat_vb: 1.0,
            stuck_flow_pct: 1.0,
            fee_spike_zscore: 0.0,
            exchange_inflow_event: false,
            cpfp_detected: false,
            congestion_regime: "normal".into(),
        },
        MempoolFeatureBar {
            schema_version: SCHEMA_VERSION.to_string(),
            ts_minute: "t2".into(),
            tx_count: 1,
            avg_fee_sat_vb: 5.0,
            p90_fee_sat_vb: 6.0,
            p99_fee_sat_vb: 8.0,
            stuck_flow_pct: 85.0,
            fee_spike_zscore: 2.0,
            exchange_inflow_event: false,
            cpfp_detected: false,
            congestion_regime: "congested".into(),
        },
    ];
    let returns = [0.01, 0.03];
    let res = run_regime_backtest(&features, &returns);
    assert!((res.sharpe_delta - (res.conditioned_mean_return - res.baseline_mean_return)).abs() < 1e-12);
}
