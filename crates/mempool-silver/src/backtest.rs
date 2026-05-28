use mempool_core::MempoolFeatureBar;

#[derive(Debug, Clone)]
pub struct BacktestSummary {
    pub baseline_mean_return: f64,
    pub conditioned_mean_return: f64,
    pub sharpe_delta: f64,
}

pub fn run_regime_backtest(features: &[MempoolFeatureBar], returns: &[f64]) -> BacktestSummary {
    let n = features.len().min(returns.len());
    if n == 0 {
        return BacktestSummary {
            baseline_mean_return: 0.0,
            conditioned_mean_return: 0.0,
            sharpe_delta: 0.0,
        };
    }
    let baseline_mean = returns[..n].iter().sum::<f64>() / n as f64;
    let mut conditioned = Vec::new();
    for i in 0..n {
        if features[i].congestion_regime == "congested" {
            conditioned.push(returns[i]);
        }
    }
    let conditioned_mean = if conditioned.is_empty() {
        0.0
    } else {
        conditioned.iter().sum::<f64>() / conditioned.len() as f64
    };
    BacktestSummary {
        baseline_mean_return: baseline_mean,
        conditioned_mean_return: conditioned_mean,
        sharpe_delta: conditioned_mean - baseline_mean,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mempool_core::SCHEMA_VERSION;

    #[test]
    fn computes_sharpe_delta() {
        let features = vec![
            MempoolFeatureBar {
                schema_version: SCHEMA_VERSION.to_string(),
                ts_minute: "t1".into(),
                tx_count: 1,
                avg_fee_sat_vb: 1.0,
                p90_fee_sat_vb: 2.0,
                p99_fee_sat_vb: 3.0,
                stuck_flow_pct: 10.0,
                fee_spike_zscore: 0.0,
                exchange_inflow_event: false,
                cpfp_detected: false,
                congestion_regime: "normal".into(),
            },
            MempoolFeatureBar {
                schema_version: SCHEMA_VERSION.to_string(),
                ts_minute: "t2".into(),
                tx_count: 1,
                avg_fee_sat_vb: 10.0,
                p90_fee_sat_vb: 20.0,
                p99_fee_sat_vb: 30.0,
                stuck_flow_pct: 80.0,
                fee_spike_zscore: 2.0,
                exchange_inflow_event: false,
                cpfp_detected: false,
                congestion_regime: "congested".into(),
            },
        ];
        let result = run_regime_backtest(&features, &[0.01, 0.03]);
        assert!(result.sharpe_delta >= 0.0);
    }
}
