use mempool_core::{bootstrap_posterior, MempoolFeatureBar, SignalPosteriorFields};

pub fn h1_posterior(history: &[MempoolFeatureBar]) -> SignalPosteriorFields {
    let samples: Vec<f64> = history
        .iter()
        .filter(|b| b.exchange_inflow_event)
        .map(|b| b.fee_spike_zscore)
        .collect();
    bootstrap_posterior(&samples, 101, "bootstrap_fee_spike_given_inflow")
}

pub fn h2_posterior(history: &[MempoolFeatureBar]) -> SignalPosteriorFields {
    let samples: Vec<f64> = history
        .iter()
        .filter(|b| b.cpfp_detected)
        .map(|b| b.stuck_flow_pct)
        .collect();
    bootstrap_posterior(&samples, 102, "bootstrap_stuck_flow_given_cpfp")
}

pub fn h3_posterior(history: &[MempoolFeatureBar]) -> SignalPosteriorFields {
    let samples: Vec<f64> = history
        .iter()
        .filter(|b| b.congestion_regime == "congested")
        .map(|b| b.fee_spike_zscore.max(0.0))
        .collect();
    bootstrap_posterior(&samples, 103, "bootstrap_fee_spike_given_congested")
}

pub fn confidence_from_posterior(p: &SignalPosteriorFields) -> f64 {
    if p.n_obs == 0 {
        return 0.0;
    }
    (p.mean.abs() / (1.0 + p.std)).clamp(0.05, 0.99)
}
