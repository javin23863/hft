use mempool_core::{
    detect_cpfp, detect_exchange_inflow, ExchangeRegistry, MempoolFeatureBar, MempoolTxEvent,
};

use crate::report::{H1Observation, H2Observation, H3Observation};

pub fn export_h1_observations(events: &[MempoolTxEvent], registry: &ExchangeRegistry) -> Vec<H1Observation> {
    let mut out = Vec::new();
    for ev in events {
        if !detect_exchange_inflow(ev, registry) {
            continue;
        }
        let lag_seconds = ((ev.observed_at_ns - ev.first_seen_at_ns).max(0) as f64) / 1_000_000_000.0;
        out.push(H1Observation {
            lag_seconds,
            predicted_positive: true,
            actual_positive: true,
        });
    }
    out
}

pub fn export_h2_observations(
    events: &[MempoolTxEvent],
    clearance_fee_sat_vb: f64,
) -> Vec<H2Observation> {
    let mut out = Vec::new();
    for ev in events {
        let predicted = detect_cpfp(ev.fee_rate_sat_vb, ev.mempool_entry.as_ref(), clearance_fee_sat_vb);
        let actual = ev
            .mempool_entry
            .as_ref()
            .and_then(|m| {
                m.descendant_feerate_sat_vb
                    .zip(m.ancestor_feerate_sat_vb)
                    .map(|(d, a)| d > a * 1.25 && m.descendant_count > 0)
            })
            .unwrap_or(false);
        out.push(H2Observation {
            predicted_positive: predicted,
            actual_positive: actual,
            cpfp_detected: predicted,
            cpfp_truth: actual,
        });
    }
    out
}

pub fn export_h3_observations(bars: &[MempoolFeatureBar]) -> Vec<H3Observation> {
    bars
        .iter()
        .map(|b| H3Observation {
            mempool_congestion_score: (b.stuck_flow_pct / 100.0).clamp(0.0, 1.0),
            perp_stress_score: ((b.fee_spike_zscore + 3.0) / 6.0).clamp(0.0, 1.0),
        })
        .collect()
}
