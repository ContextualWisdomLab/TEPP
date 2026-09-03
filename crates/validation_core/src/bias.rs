//! Signed mean bias recovery metric.

use crate::ValidationError;
use crate::input::require_paired_finite;

fn signed_residuals(truth: &[f64], recovered: &[f64]) -> Result<Vec<f64>, ValidationError> {
    require_paired_finite(truth, recovered)?;
    truth
        .iter()
        .zip(recovered)
        .map(|(truth_value, recovered_value)| {
            let residual = recovered_value - truth_value;
            if residual.is_finite() {
                Ok(residual)
            } else {
                Err(ValidationError::InvalidInput)
            }
        })
        .collect()
}

fn scaled_compensated_mean(values: &[f64]) -> Result<f64, ValidationError> {
    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }

    let mut normalized: Vec<_> = values.iter().map(|value| *value / scale).collect();
    normalized.sort_by(f64::total_cmp);

    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for value in normalized {
        let next = sum + value;
        if sum.abs() >= value.abs() {
            correction += (sum - next) + value;
        } else {
            correction += (value - next) + sum;
        }
        sum = next;
    }

    let normalized_mean = (sum + correction) / values.len() as f64;
    let mean = scale * normalized_mean;
    if !mean.is_finite() || (mean == 0.0 && normalized_mean != 0.0) {
        Err(ValidationError::InvalidInput)
    } else if mean == 0.0 {
        Ok(0.0)
    } else {
        Ok(mean)
    }
}

/// Mean signed bias `mean(recovered − truth)`.
///
/// Signed residuals are normalized by their largest magnitude and summed with
/// deterministic compensated arithmetic before the final scale is restored.
/// This keeps a representable bias from failing only because the raw residual
/// sum overflows, while a mathematically non-zero mean that falls below the
/// binary64 range fails closed rather than masquerading as zero bias.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty, unequal-length,
/// non-finite inputs, an unrepresentable signed residual, or an unrepresentable
/// final mean bias.
pub fn mean_bias(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    let residuals = signed_residuals(truth, recovered)?;
    scaled_compensated_mean(&residuals)
}

/// Standard error of the mean signed bias under independent observations.
///
/// The signed-difference mean uses the same overflow-safe deterministic
/// reference as [`mean_bias`]. The sample variance remains fail-closed when a
/// squared deviation itself is not representable.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for invalid pairs, `n < 2`, an
/// unrepresentable signed residual or mean, or non-finite variance arithmetic.
pub fn bias_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    if truth.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    let diffs = signed_residuals(truth, recovered)?;
    let mean = scaled_compensated_mean(&diffs)?;
    let mut variance_sum = 0.0_f64;
    for diff in &diffs {
        let delta = diff - mean;
        let square = delta * delta;
        if !square.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        variance_sum += square;
        if !variance_sum.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
    }
    let variance = variance_sum / (diffs.len() as f64 - 1.0);
    let standard_error = variance.sqrt() / (diffs.len() as f64).sqrt();
    if standard_error.is_finite() {
        Ok(standard_error)
    } else {
        Err(ValidationError::InvalidInput)
    }
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
        assert_eq!(se, 0.0);
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
    fn representable_extreme_constant_bias_does_not_fail_on_raw_sum_overflow() {
        assert_eq!(
            mean_bias(&[0.0, 0.0], &[f64::MAX, f64::MAX]),
            Ok(f64::MAX)
        );
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[f64::MAX, f64::MAX]),
            Ok(0.0)
        );
    }

    #[test]
    fn exact_zero_and_unrepresentable_nonzero_bias_are_distinct() {
        let ulp = f64::from_bits(1);
        assert_eq!(mean_bias(&[0.0, 0.0], &[ulp, -ulp]), Ok(0.0));
        assert_eq!(
            mean_bias(&[0.0, 0.0], &[ulp, 0.0]),
            Err(ValidationError::InvalidInput)
        );
    }

    #[test]
    fn overflow_and_nonfinite_variance_intermediates_fail_closed() {
        assert_eq!(
            mean_bias(&[-f64::MAX], &[f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[f64::MAX, -f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        let huge = 1e200;
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[huge, -huge]),
            Err(ValidationError::InvalidInput)
        );
    }
}
