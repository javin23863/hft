mod posterior;

use mempool_core::{MempoolFeatureBar, SCHEMA_VERSION, ScenarioSignal};
use mempool_core::typed_parquet::write_scenario_signals_typed;
use posterior::{confidence_from_posterior, h1_posterior, h2_posterior, h3_posterior};
use serde_json::json;

pub fn h1_exchange_inflow_signal(
    feature: &MempoolFeatureBar,
    history: &[MempoolFeatureBar],
) -> Option<ScenarioSignal> {
    if !feature.exchange_inflow_event {
        return None;
    }
    let posterior = h1_posterior(history);
    Some(ScenarioSignal {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at: feature.ts_minute.clone(),
        hypothesis_id: "H1".into(),
        signal_name: "hft_exchange_inflow_event".into(),
        confidence: confidence_from_posterior(&posterior),
        posterior,
        payload: json!({
            "regime": feature.congestion_regime,
            "tx_count": feature.tx_count
        }),
    })
}

pub fn h2_cpfp_signal(
    feature: &MempoolFeatureBar,
    history: &[MempoolFeatureBar],
) -> Option<ScenarioSignal> {
    if !feature.cpfp_detected {
        return None;
    }
    let posterior = h2_posterior(history);
    Some(ScenarioSignal {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at: feature.ts_minute.clone(),
        hypothesis_id: "H2".into(),
        signal_name: "hft_cpfp_trap_duration_sec".into(),
        confidence: confidence_from_posterior(&posterior),
        posterior,
        payload: json!({
            "stuck_flow_pct": feature.stuck_flow_pct
        }),
    })
}

pub fn h3_congestion_signal(
    feature: &MempoolFeatureBar,
    history: &[MempoolFeatureBar],
) -> Option<ScenarioSignal> {
    if feature.congestion_regime != "congested" {
        return None;
    }
    let posterior = h3_posterior(history);
    Some(ScenarioSignal {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at: feature.ts_minute.clone(),
        hypothesis_id: "H3".into(),
        signal_name: "hft_congestion_fee_stress".into(),
        confidence: confidence_from_posterior(&posterior),
        posterior,
        payload: json!({
            "p99_fee_sat_vb": feature.p99_fee_sat_vb,
            "stuck_flow_pct": feature.stuck_flow_pct,
            "fee_stress_proxy": ((feature.fee_spike_zscore + 3.0) / 6.0).clamp(0.0, 1.0)
        }),
    })
}

pub fn emit_signals_for_feature(
    feature: &MempoolFeatureBar,
    history: &[MempoolFeatureBar],
) -> Vec<ScenarioSignal> {
    let mut out = Vec::new();
    if let Some(s) = h1_exchange_inflow_signal(feature, history) {
        out.push(s);
    }
    if let Some(s) = h2_cpfp_signal(feature, history) {
        out.push(s);
    }
    if let Some(s) = h3_congestion_signal(feature, history) {
        out.push(s);
    }
    out
}

pub fn emit_signals_for_bars(bars: &[MempoolFeatureBar]) -> Vec<ScenarioSignal> {
    let mut out = Vec::new();
    for i in 0..bars.len() {
        let history = &bars[..=i];
        out.extend(emit_signals_for_feature(&bars[i], history));
    }
    out
}

pub fn write_signals_parquet(
    path: impl AsRef<std::path::Path>,
    signals: &[ScenarioSignal],
) -> anyhow::Result<()> {
    write_scenario_signals_typed(path, signals)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MempoolFeatureBar {
        MempoolFeatureBar {
            schema_version: SCHEMA_VERSION.to_string(),
            ts_minute: "2026-05-20T00:01:00Z".into(),
            tx_count: 1,
            avg_fee_sat_vb: 10.0,
            p90_fee_sat_vb: 20.0,
            p99_fee_sat_vb: 50.0,
            stuck_flow_pct: 88.0,
            fee_spike_zscore: 2.2,
            exchange_inflow_event: true,
            cpfp_detected: true,
            congestion_regime: "congested".into(),
        }
    }

    #[test]
    fn emits_all_signals_for_sample() {
        let bars = vec![sample(), sample()];
        let signals = emit_signals_for_bars(&bars);
        assert!(!signals.is_empty());
        assert!(signals.iter().all(|s| s.posterior.n_obs >= 1));
        assert!(signals.iter().all(|s| s.posterior.method.starts_with("bootstrap")));
    }
}
