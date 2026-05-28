use mempool_core::{
    confirm_exchange_inflow, cpfp_consensus_truth, detect_cpfp, has_enriched_outputs,
    predict_exchange_inflow, ExchangeRegistry, MempoolFeatureBar, MempoolTxEvent,
};

use crate::labels::GroundTruthLabels;
use crate::report::{H1Observation, H2Observation, H3Observation};

fn h1_actual(
    ev: &MempoolTxEvent,
    registry: &ExchangeRegistry,
    labels: &GroundTruthLabels,
) -> Option<bool> {
    if let Some(v) = labels.h1.get(&ev.txid) {
        return Some(*v);
    }
    if !has_enriched_outputs(ev) {
        return None;
    }
    Some(confirm_exchange_inflow(ev, registry))
}

fn h2_actual(
    ev: &MempoolTxEvent,
    clearance_fee_sat_vb: f64,
    labels: &GroundTruthLabels,
) -> Option<bool> {
    if let Some(v) = labels.h2.get(&ev.txid) {
        return Some(*v);
    }
    Some(cpfp_consensus_truth(
        ev.fee_rate_sat_vb,
        ev.mempool_entry.as_ref(),
        clearance_fee_sat_vb,
    ))
}

pub fn export_h1_observations(
    events: &[MempoolTxEvent],
    registry: &ExchangeRegistry,
    labels: &GroundTruthLabels,
    clearance_fee_sat_vb: f64,
) -> Vec<H1Observation> {
    let mut out = Vec::new();
    for ev in events {
        let predicted = predict_exchange_inflow(ev, clearance_fee_sat_vb);
        let Some(actual_positive) = h1_actual(ev, registry, labels) else {
            continue;
        };
        let lag_seconds =
            ((ev.observed_at_ns - ev.first_seen_at_ns).max(0) as f64) / 1_000_000_000.0;
        out.push(H1Observation {
            lag_seconds,
            predicted_positive: predicted,
            actual_positive,
        });
    }
    out
}

pub fn export_h2_observations(
    events: &[MempoolTxEvent],
    clearance_fee_sat_vb: f64,
    labels: &GroundTruthLabels,
) -> Vec<H2Observation> {
    let mut out = Vec::new();
    for ev in events {
        let predicted = detect_cpfp(
            ev.fee_rate_sat_vb,
            ev.mempool_entry.as_ref(),
            clearance_fee_sat_vb,
        );
        let Some(actual_positive) = h2_actual(ev, clearance_fee_sat_vb, labels) else {
            continue;
        };
        let cpfp_truth = actual_positive;
        out.push(H2Observation {
            predicted_positive: predicted,
            actual_positive,
            cpfp_detected: predicted,
            cpfp_truth,
        });
    }
    out
}

pub fn export_h3_observations(bars: &[MempoolFeatureBar]) -> Vec<H3Observation> {
    bars
        .iter()
        .map(|b| H3Observation {
            mempool_congestion_score: (b.stuck_flow_pct / 100.0).clamp(0.0, 1.0),
            fee_stress_proxy_score: ((b.fee_spike_zscore + 3.0) / 6.0).clamp(0.0, 1.0),
        })
        .collect()
}
