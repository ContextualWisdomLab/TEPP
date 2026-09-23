//! Signed and parameter-wise bias recovery metrics.

use crate::ValidationError;
use crate::input::{require_finite, require_paired_finite, slice_is_finite};

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

/// Mean signed bias for every parameter across repeated recovery observations.
///
/// `truth[j]` is the fixed true value for parameter `j`. Every row in
/// `recovered` must preserve that exact parameter geometry; the result at `j`
/// is the mean of `recovered[i][j] - truth[j]` across observations. Keeping the
/// parameter axis intact prevents simplex-wide positive and negative residuals
/// from cancelling before bias is assessed.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when truth or recovery observations
/// are empty, any recovery row changes parameter geometry, an input is
/// non-finite, or intermediate bias arithmetic becomes non-finite.
pub fn parameter_mean_biases(
    truth: &[f64],
    recovered: &[Vec<f64>],
) -> Result<Vec<f64>, ValidationError> {
    if truth.is_empty() || recovered.is_empty() || !slice_is_finite(truth) {
        return Err(ValidationError::InvalidInput);
    }

    let mut sums = vec![0.0_f64; truth.len()];
    for row in recovered {
        if row.len() != truth.len() || !slice_is_finite(row) {
            return Err(ValidationError::InvalidInput);
        }
        for ((sum, recovered_value), truth_value) in sums.iter_mut().zip(row).zip(truth) {
            let diff = recovered_value - truth_value;
            if !diff.is_finite() {
                return Err(ValidationError::InvalidInput);
            }
            *sum += diff;
            if !sum.is_finite() {
                return Err(ValidationError::InvalidInput);
            }
        }
    }

    let observation_count = recovered.len() as f64;
    sums.into_iter()
        .map(|sum| require_finite(sum / observation_count))
        .collect()
}

/// Mean absolute parameter-wise bias across repeated recovery observations.
///
/// This first computes signed bias independently for each parameter and only
/// then averages absolute magnitudes. It therefore cannot report perfect bias
/// merely because constrained parameters (for example, simplex cells) have
/// residuals that sum to zero by construction.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when
/// [`parameter_mean_biases`] rejects the recovery geometry or values, or when
/// the absolute-bias aggregation becomes non-finite.
pub fn mean_absolute_parameter_bias(
    truth: &[f64],
    recovered: &[Vec<f64>],
) -> Result<f64, ValidationError> {
    let biases = parameter_mean_biases(truth, recovered)?;
    let mut absolute_sum = 0.0_f64;
    for bias in &biases {
        absolute_sum += bias.abs();
        if !absolute_sum.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
    }
    require_finite(absolute_sum / biases.len() as f64)
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
    use super::{
        bias_standard_error, mean_absolute_parameter_bias, mean_bias, parameter_mean_biases,
    };
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
    fn parameterwise_bias_preserves_geometry_before_absolute_summary() {
        let truth = [0.6, 0.4];
        let recovered = vec![vec![0.7, 0.3], vec![0.7, 0.3], vec![0.7, 0.3]];
        let biases = parameter_mean_biases(&truth, &recovered).expect("parameter biases");
        assert!((biases[0] - 0.1).abs() < 1.0e-12);
        assert!((biases[1] + 0.1).abs() < 1.0e-12);
        assert!(biases.iter().sum::<f64>().abs() < 1.0e-12);
        assert!(
            (mean_absolute_parameter_bias(&truth, &recovered).expect("absolute bias") - 0.1).abs()
                < 1.0e-12
        );
    }

    #[test]
    fn parameterwise_bias_rejects_invalid_geometry_values_and_arithmetic() {
        assert_eq!(
            parameter_mean_biases(&[], &[vec![]]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            parameter_mean_biases(&[1.0], &[]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            parameter_mean_biases(&[f64::NAN], &[vec![1.0]]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            parameter_mean_biases(&[1.0, 2.0], &[vec![1.0]]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            parameter_mean_biases(&[1.0], &[vec![f64::INFINITY]]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            parameter_mean_biases(&[-f64::MAX], &[vec![f64::MAX]]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            parameter_mean_biases(&[0.0], &[vec![f64::MAX], vec![f64::MAX]]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            mean_absolute_parameter_bias(
                &[0.0, 0.0],
                &[vec![f64::MAX, f64::MAX]],
            ),
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
