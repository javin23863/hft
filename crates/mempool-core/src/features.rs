use crate::exchange_registry::ExchangeRegistry;
use crate::types::{MempoolEntryMeta, MempoolTxEvent};

/// Lightweight pre-enrichment predictor (fee + size); independent of address registry.
pub fn predict_exchange_inflow(ev: &MempoolTxEvent, clearance_fee_sat_vb: f64) -> bool {
    ev.fee_rate_sat_vb >= clearance_fee_sat_vb * 2.0 && ev.vsize >= 250
}

/// Confirmed inflow: requires enriched outputs and registry hit (or explicit label file).
pub fn confirm_exchange_inflow(ev: &MempoolTxEvent, registry: &ExchangeRegistry) -> bool {
    if ev.output_addresses.is_empty() {
        return false;
    }
    registry.tx_has_exchange_output(ev.output_addresses.iter())
}

pub fn has_enriched_outputs(ev: &MempoolTxEvent) -> bool {
    ev.output_addresses.iter().any(|a| a.is_some())
}

/// Operational CPFP detector used for signals and H2 predicted side.
pub fn detect_cpfp(
    fee_rate_sat_vb: f64,
    meta: Option<&MempoolEntryMeta>,
    clearance_fee_sat_vb: f64,
) -> bool {
    let Some(meta) = meta else {
        return false;
    };
    if meta.descendant_count == 0 {
        return false;
    }
    if fee_rate_sat_vb < clearance_fee_sat_vb * 1.5 {
        return false;
    }
    if let (Some(anc), Some(desc)) = (
        meta.ancestor_feerate_sat_vb,
        meta.descendant_feerate_sat_vb,
    ) {
        return desc > anc * 1.25;
    }
    meta.ancestor_count > 0
}

/// Stricter package definition for H2 ground truth (independent thresholds).
pub fn cpfp_consensus_truth(
    fee_rate_sat_vb: f64,
    meta: Option<&MempoolEntryMeta>,
    clearance_fee_sat_vb: f64,
) -> bool {
    let Some(meta) = meta else {
        return false;
    };
    if meta.descendant_count < 2 {
        return false;
    }
    if fee_rate_sat_vb < clearance_fee_sat_vb * 2.0 {
        return false;
    }
    match (
        meta.ancestor_feerate_sat_vb,
        meta.descendant_feerate_sat_vb,
    ) {
        (Some(anc), Some(desc)) => desc > anc * 1.5,
        _ => false,
    }
}

pub fn detect_exchange_inflow(event: &MempoolTxEvent, registry: &ExchangeRegistry) -> bool {
    confirm_exchange_inflow(event, registry)
}

pub fn aggregate_bar_flags(events: &[MempoolTxEvent], registry: &ExchangeRegistry, clearance: f64) -> (bool, bool) {
    let mut inflow = false;
    let mut cpfp = false;
    for ev in events {
        if confirm_exchange_inflow(ev, registry) {
            inflow = true;
        }
        if detect_cpfp(ev.fee_rate_sat_vb, ev.mempool_entry.as_ref(), clearance) {
            cpfp = true;
        }
    }
    (inflow, cpfp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MempoolEntryMeta;
    use std::collections::HashSet;

    fn registry_with(addr: &str) -> ExchangeRegistry {
        ExchangeRegistry::from_addresses(HashSet::from([addr.to_string()]))
    }

    #[test]
    fn cpfp_requires_descendants_and_elevated_fee() {
        let meta = MempoolEntryMeta {
            ancestor_count: 1,
            descendant_count: 1,
            ancestor_feerate_sat_vb: Some(2.0),
            descendant_feerate_sat_vb: Some(20.0),
        };
        assert!(detect_cpfp(20.0, Some(&meta), 10.0));
        assert!(!detect_cpfp(5.0, Some(&meta), 10.0));
    }

    #[test]
    fn consensus_truth_stricter_than_detector() {
        let meta = MempoolEntryMeta {
            ancestor_count: 1,
            descendant_count: 1,
            ancestor_feerate_sat_vb: Some(2.0),
            descendant_feerate_sat_vb: Some(20.0),
        };
        assert!(detect_cpfp(20.0, Some(&meta), 10.0));
        assert!(!cpfp_consensus_truth(20.0, Some(&meta), 10.0));
    }

    #[test]
    fn predict_and_confirm_are_independent() {
        let reg = registry_with("bc1qdep");
        let ev = MempoolTxEvent {
            schema_version: "1.0.0".into(),
            observed_at_ns: 0,
            txid: "a".into(),
            first_seen_at_ns: 0,
            fee_rate_sat_vb: 25.0,
            vsize: 400,
            rbf_signaling: false,
            output_values_sats: vec![],
            output_addresses: vec![Some("bc1qdep".into())],
            mempool_entry: None,
            node_sync_height: 0,
            node_ibd_complete: true,
            source_node_id: "n".into(),
        };
        assert!(predict_exchange_inflow(&ev, 10.0));
        assert!(confirm_exchange_inflow(&ev, &reg));
    }
}
