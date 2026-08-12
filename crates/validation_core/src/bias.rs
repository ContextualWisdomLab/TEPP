//! Signed mean bias recovery metric.

use crate::ValidationError;
use crate::input::require_paired_finite;

/// Mean signed bias `mean(recovered − truth)`.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty, unequal-length, or
/// non-finite inputs.
pub fn mean_bias(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    require_paired_finite(truth, recovered)?;
    let sum: f64 = truth.iter().zip(recovered).map(|(t, r)| r - t).sum();
    Ok(sum / truth.len() as f64)
}

/// Standard error of the mean signed bias under independent observations.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for invalid pairs or `n < 2`.
pub fn bias_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    if truth.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    require_paired_finite(truth, recovered)?;
    let diffs: Vec<f64> = truth.iter().zip(recovered).map(|(t, r)| r - t).collect();
    let mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
    let variance = diffs
        .iter()
        .map(|diff| {
            let delta = diff - mean;
            delta * delta
        })
        .sum::<f64>()
        / (diffs.len() as f64 - 1.0);
    Ok(variance.sqrt() / (diffs.len() as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::{bias_standard_error, mean_bias};
    use crate::ValidationError;

    #[test]
    fn bias_oracle_and_degenerate_cases() {
        let truth = [1.0, 2.0, 3.0];
        let recovered = [2.0, 3.0, 4.0];
        assert!((mean_bias(&truth, &recovered).expect("bias") - 1.0).abs() < 1e-12);
        let se = bias_standard_error(&truth, &recovered).expect("se");
        assert!((se - 0.0).abs() < 1e-12);
        assert_eq!(mean_bias(&[], &[]), Err(ValidationError::InvalidInput));
        assert_eq!(
            mean_bias(&[1.0], &[1.0, 2.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            mean_bias(&[f64::INFINITY], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            bias_standard_error(&[1.0], &[2.0]),
            Err(ValidationError::InvalidInput)
        );
        let se_var = bias_standard_error(&[0.0, 0.0], &[1.0, -1.0]).expect("se");
        assert!(se_var > 0.0);
    }
}
