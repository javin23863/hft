use feasibility::{export_h1_observations, export_h2_observations, GroundTruthLabels};
use mempool_core::types::{MempoolEntryMeta, MempoolTxEvent};
use mempool_core::{ExchangeRegistry, SCHEMA_VERSION};
use std::collections::HashSet;

#[test]
fn h1_export_includes_negative_labeled_rows() {
    let reg = ExchangeRegistry::from_addresses(HashSet::new());
    let mut labels = GroundTruthLabels::default();
    labels.h1.insert("tx-fp".into(), false);

    let events = vec![MempoolTxEvent {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at_ns: 0,
        txid: "tx-fp".into(),
        first_seen_at_ns: 0,
        fee_rate_sat_vb: 30.0,
        vsize: 400,
        rbf_signaling: false,
        output_values_sats: vec![],
        output_addresses: vec![Some("bc1qunknown".into())],
        mempool_entry: None,
        node_sync_height: 0,
        node_ibd_complete: true,
        source_node_id: "n".into(),
    }];

    let rows = export_h1_observations(&events, &reg, &labels, 10.0);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].predicted_positive);
    assert!(!rows[0].actual_positive);
}

#[test]
fn h2_predicted_and_truth_can_diverge() {
    let labels = GroundTruthLabels::default();
    let events = vec![MempoolTxEvent {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at_ns: 0,
        txid: "tx-cpfp".into(),
        first_seen_at_ns: 0,
        fee_rate_sat_vb: 20.0,
        vsize: 200,
        rbf_signaling: false,
        output_values_sats: vec![],
        output_addresses: vec![],
        mempool_entry: Some(MempoolEntryMeta {
            ancestor_count: 1,
            descendant_count: 1,
            ancestor_feerate_sat_vb: Some(2.0),
            descendant_feerate_sat_vb: Some(20.0),
        }),
        node_sync_height: 0,
        node_ibd_complete: true,
        source_node_id: "n".into(),
    }];
    let rows = export_h2_observations(&events, 10.0, &labels);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].predicted_positive);
    assert!(!rows[0].actual_positive);
}
