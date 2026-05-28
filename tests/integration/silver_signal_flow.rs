use mempool_core::{FeeSnapshot, SCHEMA_VERSION};
use mempool_silver::build_feature_bars;
use scenario_engines::{h1_exchange_inflow_signal, h2_cpfp_signal, h3_congestion_signal};

#[test]
fn silver_to_scenario_flow() {
    let snaps = vec![FeeSnapshot {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at: "2026-05-20T00:00:00Z".into(),
        size_txs: 1200,
        bytes: 1_000_000,
        min_fee_sat_vb: 3.0,
        p50_fee_sat_vb: 4.0,
        p90_fee_sat_vb: 20.0,
        p99_fee_sat_vb: 120.0,
        stuck_flow_pct: 90.0,
        source_node_id: "n1".into(),
    }];

    let mut bars = build_feature_bars(&snaps);
    bars[0].exchange_inflow_event = true;
    bars[0].cpfp_detected = true;
    bars[0].congestion_regime = "congested".into();
    assert!(h1_exchange_inflow_signal(&bars[0]).is_some());
    assert!(h2_cpfp_signal(&bars[0]).is_some());
    assert!(h3_congestion_signal(&bars[0]).is_some());
}
