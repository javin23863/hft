use crate::exchange_registry::ExchangeRegistry;
use crate::types::{MempoolEntryMeta, MempoolTxEvent};

/// CPFP heuristic: package has descendants and this tx pays materially above clearance.
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

pub fn detect_exchange_inflow(event: &MempoolTxEvent, registry: &ExchangeRegistry) -> bool {
    registry.tx_has_exchange_output(event.output_addresses.iter())
}

pub fn aggregate_bar_flags(events: &[MempoolTxEvent], registry: &ExchangeRegistry, clearance: f64) -> (bool, bool) {
    let mut inflow = false;
    let mut cpfp = false;
    for ev in events {
        if detect_exchange_inflow(ev, registry) {
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
    fn exchange_inflow_matches_output() {
        let reg = registry_with("bc1qdep");
        let ev = MempoolTxEvent {
            schema_version: "1.0.0".into(),
            observed_at_ns: 0,
            txid: "a".into(),
            first_seen_at_ns: 0,
            fee_rate_sat_vb: 1.0,
            vsize: 100,
            rbf_signaling: false,
            output_values_sats: vec![],
            output_addresses: vec![Some("bc1qdep".into())],
            mempool_entry: None,
            node_sync_height: 0,
            node_ibd_complete: true,
            source_node_id: "n".into(),
        };
        assert!(detect_exchange_inflow(&ev, &reg));
    }
}
