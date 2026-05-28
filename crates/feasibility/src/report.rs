use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeasibilityOutcome {
    Keep,
    Kill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H1Observation {
    pub lag_seconds: f64,
    pub predicted_positive: bool,
    pub actual_positive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Observation {
    pub predicted_positive: bool,
    pub actual_positive: bool,
    pub cpfp_detected: bool,
    pub cpfp_truth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H3Observation {
    pub mempool_congestion_score: f64,
    pub fee_stress_proxy_score: f64,
}

#[derive(Debug, Clone)]
pub struct MetricEstimate {
    pub value: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub n_obs: usize,
}

#[derive(Debug, Clone)]
pub struct FeasibilityReport {
    pub h1: FeasibilityOutcome,
    pub h2: FeasibilityOutcome,
    pub h3: FeasibilityOutcome,
    pub h1_lag_median: MetricEstimate,
    pub h1_precision: MetricEstimate,
    pub h2_fpr: MetricEstimate,
    pub h2_cpfp_accuracy: MetricEstimate,
    pub h3_corr: MetricEstimate,
    pub notes: String,
}

fn percentile(values: &mut [f64], q: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((values.len() as f64 - 1.0) * q).round() as usize;
    values[idx.min(values.len() - 1)]
}

fn bootstrap_ci<F>(n: usize, seed: u64, mut stat: F) -> (f64, f64)
where
    F: FnMut(&mut ChaCha8Rng) -> f64,
{
    if n == 0 {
        return (0.0, 0.0);
    }
    let reps = 1_000;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut samples = Vec::with_capacity(reps);
    for _ in 0..reps {
        samples.push(stat(&mut rng));
    }
    let low = percentile(&mut samples.clone(), 0.025);
    let high = percentile(&mut samples, 0.975);
    (low, high)
}

fn sample_with_replacement<'a, T>(src: &'a [T], rng: &mut ChaCha8Rng) -> Vec<&'a T> {
    (0..src.len())
        .map(|_| {
            let idx = rng.gen_range(0..src.len());
            &src[idx]
        })
        .collect()
}

pub fn estimate_h1_lag(observations: &[H1Observation]) -> MetricEstimate {
    let mut lags: Vec<f64> = observations.iter().map(|o| o.lag_seconds).collect();
    let value = percentile(&mut lags, 0.5);
    let (ci_low, ci_high) = bootstrap_ci(observations.len(), 7, |rng| {
        let mut sampled = sample_with_replacement(observations, rng)
            .iter()
            .map(|o| o.lag_seconds)
            .collect::<Vec<_>>();
        percentile(&mut sampled, 0.5)
    });
    MetricEstimate {
        value,
        ci_low,
        ci_high,
        n_obs: observations.len(),
    }
}

pub fn estimate_h1_precision(observations: &[H1Observation]) -> MetricEstimate {
    let tp = observations
        .iter()
        .filter(|o| o.predicted_positive && o.actual_positive)
        .count() as f64;
    let fp = observations
        .iter()
        .filter(|o| o.predicted_positive && !o.actual_positive)
        .count() as f64;
    let value = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
    let (ci_low, ci_high) = bootstrap_ci(observations.len(), 11, |rng| {
        let sampled = sample_with_replacement(observations, rng);
        let tp_s = sampled
            .iter()
            .filter(|o| o.predicted_positive && o.actual_positive)
            .count() as f64;
        let fp_s = sampled
            .iter()
            .filter(|o| o.predicted_positive && !o.actual_positive)
            .count() as f64;
        if tp_s + fp_s > 0.0 {
            tp_s / (tp_s + fp_s)
        } else {
            0.0
        }
    });
    MetricEstimate {
        value,
        ci_low,
        ci_high,
        n_obs: observations.len(),
    }
}

pub fn estimate_h2_fpr(observations: &[H2Observation]) -> MetricEstimate {
    let fp = observations
        .iter()
        .filter(|o| o.predicted_positive && !o.actual_positive)
        .count() as f64;
    let tn = observations
        .iter()
        .filter(|o| !o.predicted_positive && !o.actual_positive)
        .count() as f64;
    let value = if fp + tn > 0.0 { fp / (fp + tn) } else { 0.0 };
    let (ci_low, ci_high) = bootstrap_ci(observations.len(), 13, |rng| {
        let sampled = sample_with_replacement(observations, rng);
        let fp_s = sampled
            .iter()
            .filter(|o| o.predicted_positive && !o.actual_positive)
            .count() as f64;
        let tn_s = sampled
            .iter()
            .filter(|o| !o.predicted_positive && !o.actual_positive)
            .count() as f64;
        if fp_s + tn_s > 0.0 {
            fp_s / (fp_s + tn_s)
        } else {
            0.0
        }
    });
    MetricEstimate {
        value,
        ci_low,
        ci_high,
        n_obs: observations.len(),
    }
}

pub fn estimate_h2_accuracy(observations: &[H2Observation]) -> MetricEstimate {
    let correct = observations
        .iter()
        .filter(|o| o.cpfp_detected == o.cpfp_truth)
        .count() as f64;
    let n = observations.len() as f64;
    let value = if n > 0.0 { correct / n } else { 0.0 };
    let (ci_low, ci_high) = bootstrap_ci(observations.len(), 17, |rng| {
        let sampled = sample_with_replacement(observations, rng);
        let c = sampled
            .iter()
            .filter(|o| o.cpfp_detected == o.cpfp_truth)
            .count() as f64;
        c / sampled.len().max(1) as f64
    });
    MetricEstimate {
        value,
        ci_low,
        ci_high,
        n_obs: observations.len(),
    }
}

pub fn estimate_h3_corr(observations: &[H3Observation]) -> MetricEstimate {
    fn corr(xs: &[f64], ys: &[f64]) -> f64 {
        if xs.len() != ys.len() || xs.len() <= 1 {
            return 0.0;
        }
        let mx = xs.iter().sum::<f64>() / xs.len() as f64;
        let my = ys.iter().sum::<f64>() / ys.len() as f64;
        let mut num = 0.0;
        let mut dx = 0.0;
        let mut dy = 0.0;
        for (x, y) in xs.iter().zip(ys.iter()) {
            let vx = x - mx;
            let vy = y - my;
            num += vx * vy;
            dx += vx * vx;
            dy += vy * vy;
        }
        let denom = (dx * dy).sqrt();
        if denom > 0.0 { num / denom } else { 0.0 }
    }

    let xs = observations
        .iter()
        .map(|o| o.mempool_congestion_score)
        .collect::<Vec<_>>();
    let ys = observations
        .iter()
        .map(|o| o.fee_stress_proxy_score)
        .collect::<Vec<_>>();
    let value = corr(&xs, &ys);
    let (ci_low, ci_high) = bootstrap_ci(observations.len(), 19, |rng| {
        let sampled = sample_with_replacement(observations, rng);
        let sx = sampled
            .iter()
            .map(|o| o.mempool_congestion_score)
            .collect::<Vec<_>>();
        let sy = sampled
            .iter()
            .map(|o| o.fee_stress_proxy_score)
            .collect::<Vec<_>>();
        corr(&sx, &sy)
    });
    MetricEstimate {
        value,
        ci_low,
        ci_high,
        n_obs: observations.len(),
    }
}

pub fn build_report(
    h1_obs: &[H1Observation],
    h2_obs: &[H2Observation],
    h3_obs: &[H3Observation],
) -> FeasibilityReport {
    let h1_lag_median = estimate_h1_lag(h1_obs);
    let h1_precision = estimate_h1_precision(h1_obs);
    let h2_fpr = estimate_h2_fpr(h2_obs);
    let h2_cpfp_accuracy = estimate_h2_accuracy(h2_obs);
    let h3_corr = estimate_h3_corr(h3_obs);

    let h1 = if h1_lag_median.value < 2.0 || h1_precision.value < 0.80 {
        FeasibilityOutcome::Kill
    } else {
        FeasibilityOutcome::Keep
    };
    let h2 = if h2_fpr.value > 0.30 || h2_cpfp_accuracy.value < 0.90 {
        FeasibilityOutcome::Kill
    } else {
        FeasibilityOutcome::Keep
    };
    let h3 = if h3_corr.value < 0.15 {
        FeasibilityOutcome::Kill
    } else {
        FeasibilityOutcome::Keep
    };

    FeasibilityReport {
        h1,
        h2,
        h3,
        h1_lag_median,
        h1_precision,
        h2_fpr,
        h2_cpfp_accuracy,
        h3_corr,
        notes: "Measured estimators with deterministic bootstrap CI. H1 rows require enriched outputs or explicit ground-truth labels. H2 actual uses stricter cpfp_consensus_truth than the operational detector. H3 fee_stress_proxy is a mempool z-score proxy (not external perp market data).".to_string(),
    }
}

pub fn write_report_markdown(path: impl AsRef<Path>, report: &FeasibilityReport) -> Result<()> {
    let as_str = |x| match x {
        FeasibilityOutcome::Keep => "KEEP",
        FeasibilityOutcome::Kill => "KILL",
    };

    let content = format!(
        "# Feasibility Report\n\n\
## Outcomes\n\
- H1: {}\n\
- H2: {}\n\
- H3: {}\n\n\
## Metrics\n\
- H1 lag median (sec): {:.4} [{:.4}, {:.4}] (n={})\n\
- H1 precision TP/(TP+FP): {:.4} [{:.4}, {:.4}] (n={})\n\
- H2 false-positive rate FP/(FP+TN): {:.4} [{:.4}, {:.4}] (n={})\n\
- H2 CPFP detection accuracy: {:.4} [{:.4}, {:.4}] (n={})\n\
- H3 correlation(congestion, fee_stress_proxy): {:.4} [{:.4}, {:.4}] (n={})\n\n\
Notes: {}\n",
        as_str(report.h1),
        as_str(report.h2),
        as_str(report.h3),
        report.h1_lag_median.value,
        report.h1_lag_median.ci_low,
        report.h1_lag_median.ci_high,
        report.h1_lag_median.n_obs,
        report.h1_precision.value,
        report.h1_precision.ci_low,
        report.h1_precision.ci_high,
        report.h1_precision.n_obs,
        report.h2_fpr.value,
        report.h2_fpr.ci_low,
        report.h2_fpr.ci_high,
        report.h2_fpr.n_obs,
        report.h2_cpfp_accuracy.value,
        report.h2_cpfp_accuracy.ci_low,
        report.h2_cpfp_accuracy.ci_high,
        report.h2_cpfp_accuracy.n_obs,
        report.h3_corr.value,
        report.h3_corr.ci_low,
        report.h3_corr.ci_high,
        report.h3_corr.n_obs,
        report.notes
    );
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(path.as_ref(), content).context("write feasibility markdown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_report_is_deterministic() {
        let h1 = vec![
            H1Observation {
                lag_seconds: 3.0,
                predicted_positive: true,
                actual_positive: true,
            },
            H1Observation {
                lag_seconds: 4.0,
                predicted_positive: true,
                actual_positive: false,
            },
            H1Observation {
                lag_seconds: 2.5,
                predicted_positive: true,
                actual_positive: true,
            },
        ];
        let h2 = vec![
            H2Observation {
                predicted_positive: true,
                actual_positive: true,
                cpfp_detected: true,
                cpfp_truth: true,
            },
            H2Observation {
                predicted_positive: false,
                actual_positive: false,
                cpfp_detected: false,
                cpfp_truth: false,
            },
            H2Observation {
                predicted_positive: true,
                actual_positive: false,
                cpfp_detected: true,
                cpfp_truth: true,
            },
        ];
        let h3 = vec![
            H3Observation {
                mempool_congestion_score: 0.1,
                fee_stress_proxy_score: 0.1,
            },
            H3Observation {
                mempool_congestion_score: 0.5,
                fee_stress_proxy_score: 0.45,
            },
            H3Observation {
                mempool_congestion_score: 0.9,
                fee_stress_proxy_score: 0.8,
            },
        ];

        let r1 = build_report(&h1, &h2, &h3);
        let r2 = build_report(&h1, &h2, &h3);
        assert_eq!(r1.h1_lag_median.value, r2.h1_lag_median.value);
        assert_eq!(r1.h1_precision.value, r2.h1_precision.value);
        assert_eq!(r1.h2_fpr.value, r2.h2_fpr.value);
        assert_eq!(r1.h3_corr.value, r2.h3_corr.value);
    }
}
