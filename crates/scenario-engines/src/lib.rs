use mempool_core::parquet_io::write_json_rows_parquet;
use mempool_core::{MempoolFeatureBar, ScenarioSignal, SignalPosteriorFields, SCHEMA_VERSION};
use serde_json::json;

pub fn h1_exchange_inflow_signal(feature: &MempoolFeatureBar) -> Option<ScenarioSignal> {
    if !feature.exchange_inflow_event {
        return None;
    }
    Some(ScenarioSignal {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at: feature.ts_minute.clone(),
        hypothesis_id: "H1".into(),
        signal_name: "hft_exchange_inflow_event".into(),
        confidence: 0.6,
        posterior: SignalPosteriorFields {
            mean: feature.fee_spike_zscore,
            std: 1.0,
            n_obs: 1,
            method: "bootstrap_placeholder".into(),
        },
        payload: json!({
            "regime": feature.congestion_regime,
            "tx_count": feature.tx_count
        }),
    })
}

pub fn h2_cpfp_signal(feature: &MempoolFeatureBar) -> Option<ScenarioSignal> {
    if !feature.cpfp_detected {
        return None;
    }
    Some(ScenarioSignal {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at: feature.ts_minute.clone(),
        hypothesis_id: "H2".into(),
        signal_name: "hft_cpfp_trap_duration_sec".into(),
        confidence: 0.55,
        posterior: SignalPosteriorFields {
            mean: feature.stuck_flow_pct,
            std: 0.8,
            n_obs: 1,
            method: "bootstrap_placeholder".into(),
        },
        payload: json!({
            "stuck_flow_pct": feature.stuck_flow_pct
        }),
    })
}

pub fn h3_congestion_signal(feature: &MempoolFeatureBar) -> Option<ScenarioSignal> {
    if feature.congestion_regime != "congested" {
        return None;
    }
    Some(ScenarioSignal {
        schema_version: SCHEMA_VERSION.to_string(),
        observed_at: feature.ts_minute.clone(),
        hypothesis_id: "H3".into(),
        signal_name: "hft_congestion_perp_stress".into(),
        confidence: 0.5,
        posterior: SignalPosteriorFields {
            mean: feature.fee_spike_zscore.max(0.0),
            std: 0.9,
            n_obs: 1,
            method: "bootstrap_placeholder".into(),
        },
        payload: json!({
            "p99_fee_sat_vb": feature.p99_fee_sat_vb,
            "stuck_flow_pct": feature.stuck_flow_pct
        }),
    })
}

pub fn emit_signals_for_feature(feature: &MempoolFeatureBar) -> Vec<ScenarioSignal> {
    let mut out = Vec::new();
    if let Some(s) = h1_exchange_inflow_signal(feature) {
        out.push(s);
    }
    if let Some(s) = h2_cpfp_signal(feature) {
        out.push(s);
    }
    if let Some(s) = h3_congestion_signal(feature) {
        out.push(s);
    }
    out
}

pub fn write_signals_parquet(
    path: impl AsRef<std::path::Path>,
    signals: &[ScenarioSignal],
) -> anyhow::Result<()> {
    write_json_rows_parquet(path, signals)
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
        let f = sample();
        assert!(h1_exchange_inflow_signal(&f).is_some());
        assert!(h2_cpfp_signal(&f).is_some());
        assert!(h3_congestion_signal(&f).is_some());
        assert_eq!(emit_signals_for_feature(&f).len(), 3);
    }
}
