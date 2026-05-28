use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::types::SignalPosteriorFields;

/// Bootstrap mean and std of the mean over `samples` (deterministic seed).
pub fn bootstrap_posterior(samples: &[f64], seed: u64, method: &str) -> SignalPosteriorFields {
    let n = samples.len();
    if n == 0 {
        return SignalPosteriorFields {
            mean: 0.0,
            std: 0.0,
            n_obs: 0,
            method: method.to_string(),
        };
    }
    let mean = samples.iter().sum::<f64>() / n as f64;
    if n == 1 {
        return SignalPosteriorFields {
            mean,
            std: 0.0,
            n_obs: 1,
            method: method.to_string(),
        };
    }

    let reps = 400usize.min(50 * n);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let mut boot_means = Vec::with_capacity(reps);
    for _ in 0..reps {
        let mut sum = 0.0;
        for _ in 0..n {
            let idx = rng.gen_range(0..n);
            sum += samples[idx];
        }
        boot_means.push(sum / n as f64);
    }
    boot_means.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let std = {
        let m = boot_means.iter().sum::<f64>() / boot_means.len() as f64;
        let v = boot_means
            .iter()
            .map(|x| {
                let d = x - m;
                d * d
            })
            .sum::<f64>()
            / boot_means.len() as f64;
        v.sqrt()
    };

    SignalPosteriorFields {
        mean,
        std,
        n_obs: n as u64,
        method: method.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posterior_uses_sample_count() {
        let p = bootstrap_posterior(&[1.0, 2.0, 3.0, 4.0], 1, "bootstrap_mean");
        assert_eq!(p.n_obs, 4);
        assert!((p.mean - 2.5).abs() < 1e-9);
        assert!(p.std >= 0.0);
    }
}
