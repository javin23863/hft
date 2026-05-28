use mempool_core::types::MempoolEntryMeta;
use mempool_core::{FeeSnapshot, MempoolTxEvent, SCHEMA_VERSION};
use mempool_silver::build_feature_bars;
use scenario_engines::{emit_signals_for_bars, h1_exchange_inflow_signal, h2_cpfp_signal, h3_congestion_signal};
use std::collections::HashSet;
use mempool_core::ExchangeRegistry;

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

    let events = vec![MempoolTxEvent {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at_ns: 1_700_000_000_000_000_000,
        txid: "tx1".into(),
        first_seen_at_ns: 1_700_000_000_000_000_000,
        fee_rate_sat_vb: 30.0,
        vsize: 200,
        rbf_signaling: false,
        output_values_sats: vec![],
        output_addresses: vec![Some("bc1qdep".into())],
        mempool_entry: Some(MempoolEntryMeta {
            ancestor_count: 1,
            descendant_count: 1,
            ancestor_feerate_sat_vb: Some(2.0),
            descendant_feerate_sat_vb: Some(30.0),
        }),
        node_sync_height: 0,
        node_ibd_complete: true,
        source_node_id: "n1".into(),
    }];
    let reg = ExchangeRegistry::from_addresses(HashSet::from(["bc1qdep".to_string()]));
    let mut bars = build_feature_bars(&snaps, &events, &reg, 10.0);
    bars[0].congestion_regime = "congested".into();
    assert!(h1_exchange_inflow_signal(&bars[0], &bars).is_some());
    assert!(h2_cpfp_signal(&bars[0], &bars).is_some());
    assert!(h3_congestion_signal(&bars[0]).is_some());
}
