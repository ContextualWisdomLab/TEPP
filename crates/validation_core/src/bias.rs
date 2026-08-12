//! Signed mean bias recovery metric.

use crate::ValidationError;
use crate::input::{require_finite, require_paired_finite};

/// Mean signed bias `mean(recovered − truth)`.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty, unequal-length,
/// non-finite inputs, or arithmetic overflow to a non-finite mean.
pub fn mean_bias(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    require_paired_finite(truth, recovered)?;
    let mut sum = 0.0_f64;
    for (t, r) in truth.iter().zip(recovered) {
        let diff = r - t;
        if !diff.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        sum += diff;
        if !sum.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
    }
    require_finite(sum / truth.len() as f64)
}

/// Standard error of the mean signed bias under independent observations.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for invalid pairs, `n < 2`, or
/// non-finite intermediate bias arithmetic.
pub fn bias_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    if truth.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    require_paired_finite(truth, recovered)?;
    let mut diffs = Vec::with_capacity(truth.len());
    for (t, r) in truth.iter().zip(recovered) {
        let diff = r - t;
        if !diff.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        diffs.push(diff);
    }
    let mean = require_finite(diffs.iter().sum::<f64>() / diffs.len() as f64)?;
    let mut variance_sum = 0.0_f64;
    for diff in &diffs {
        let delta = diff - mean;
        let square = delta * delta;
        if !square.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        variance_sum += square;
    }
    let variance = variance_sum / (diffs.len() as f64 - 1.0);
    require_finite(require_finite(variance.sqrt())? / (diffs.len() as f64).sqrt())
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
        assert_eq!(
            mean_bias(&[f64::MAX], &[-f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            bias_standard_error(&[f64::MAX, 0.0], &[-f64::MAX, 0.0]),
            Err(ValidationError::InvalidInput)
        );
    }

    #[test]
    fn overflow_and_nonfinite_intermediates_fail_closed() {
        assert_eq!(
            mean_bias(&[0.0, 0.0], &[f64::MAX, f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            mean_bias(&[-f64::MAX], &[f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[f64::MAX, -f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        // Squared deviation overflows for extreme residuals.
        let huge = 1e200;
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[huge, -huge]),
            Err(ValidationError::InvalidInput)
        );
    }
}
