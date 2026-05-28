use feasibility::{
    build_report, export_h1_observations, export_h2_observations, export_h3_observations,
    write_report_markdown, GroundTruthLabels,
};
use mempool_core::types::{MempoolEntryMeta, MempoolTxEvent};
use mempool_core::{ExchangeRegistry, FeeSnapshot, SCHEMA_VERSION};
use mempool_silver::build_feature_bars;
use scenario_engines::emit_signals_for_bars;
use std::collections::HashSet;

#[test]
fn bronze_to_feasibility_report_pipeline() {
    let reg = ExchangeRegistry::from_addresses(HashSet::from(["bc1qdep".to_string()]));
    let labels = GroundTruthLabels::default();
    let clearance = 10.0;

    let snaps = vec![
        FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: "2026-05-20T00:00:00Z".into(),
            size_txs: 100,
            bytes: 10_000,
            min_fee_sat_vb: 2.0,
            p50_fee_sat_vb: 5.0,
            p90_fee_sat_vb: 15.0,
            p99_fee_sat_vb: 40.0,
            stuck_flow_pct: 75.0,
            source_node_id: "n1".into(),
        },
        FeeSnapshot {
            schema_version: SCHEMA_VERSION.to_string(),
            observed_at: "2026-05-20T00:01:00Z".into(),
            size_txs: 200,
            bytes: 20_000,
            min_fee_sat_vb: 3.0,
            p50_fee_sat_vb: 8.0,
            p90_fee_sat_vb: 25.0,
            p99_fee_sat_vb: 90.0,
            stuck_flow_pct: 85.0,
            source_node_id: "n1".into(),
        },
    ];

    let events = vec![MempoolTxEvent {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at_ns: 1_771_526_400_000_000_000,
        txid: "tx1".into(),
        first_seen_at_ns: 1_771_526_399_000_000_000,
        fee_rate_sat_vb: 30.0,
        vsize: 400,
        rbf_signaling: false,
        output_values_sats: vec![],
        output_addresses: vec![Some("bc1qdep".into())],
        mempool_entry: Some(MempoolEntryMeta {
            ancestor_count: 1,
            descendant_count: 2,
            ancestor_feerate_sat_vb: Some(2.0),
            descendant_feerate_sat_vb: Some(30.0),
        }),
        node_sync_height: 0,
        node_ibd_complete: true,
        source_node_id: "n1".into(),
    }];

    let bars = build_feature_bars(&snaps, &events, &reg, clearance);
    let signals = emit_signals_for_bars(&bars);
    assert!(!signals.is_empty());

    let h1 = export_h1_observations(&events, &reg, &labels, clearance);
    let h2 = export_h2_observations(&events, clearance, &labels);
    let h3 = export_h3_observations(&bars);
    let report = build_report(&h1, &h2, &h3);
    assert!(report.h1_lag_median.n_obs > 0 || h1.is_empty());
    let md = {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("report.md");
        write_report_markdown(&path, &report).unwrap();
        std::fs::read_to_string(path).unwrap()
    };
    assert!(md.contains("H3 correlation(congestion, fee_stress_proxy)"));
}
