//! Signed mean bias recovery metric.

use crate::ValidationError;
use crate::input::require_paired_finite;
use crate::numeric::deterministic_compensated_sum;

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

fn exact_power_of_two_scale(max_magnitude: f64) -> f64 {
    let bits = max_magnitude.to_bits();
    let exponent = (bits >> 52) & 0x7ff;
    if exponent == 0 {
        let significand = bits & 0x000f_ffff_ffff_ffff;
        let highest_bit = 63 - significand.leading_zeros();
        f64::from_bits(1_u64 << highest_bit)
    } else {
        f64::from_bits(exponent << 52)
    }
}

fn same_sign_mean_over_total(values: &[f64], total_count: usize) -> Result<f64, ValidationError> {
    let max_magnitude = values
        .iter()
        .map(|value| value.abs())
        .max_by(f64::total_cmp)
        .ok_or(ValidationError::InvalidInput)?;
    if max_magnitude == 0.0 {
        return Ok(0.0);
    }

    let scale = exact_power_of_two_scale(max_magnitude);
    let normalized = values.iter().map(|value| *value / scale).collect();
    let normalized_mean = deterministic_compensated_sum(normalized) / total_count as f64;
    let mean = normalized_mean * scale;
    if !mean.is_finite() || mean == 0.0 {
        Err(ValidationError::InvalidInput)
    } else {
        Ok(mean)
    }
}

fn scaled_compensated_mean(values: &[f64]) -> Result<f64, ValidationError> {
    let mut positives = Vec::new();
    let mut negatives = Vec::new();
    for &value in values {
        if !value.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        if value > 0.0 {
            positives.push(value);
        } else if value < 0.0 {
            negatives.push(value);
        }
    }

    if positives.is_empty() && negatives.is_empty() {
        return Ok(0.0);
    }
    if positives.is_empty() || negatives.is_empty() {
        return same_sign_mean_over_total(values, values.len());
    }

    positives.sort_by(|left, right| right.total_cmp(left));
    negatives.sort_by(|left, right| left.total_cmp(right));

    let mut positive_index = 0_usize;
    let mut negative_index = 0_usize;
    let mut positive = positives[0];
    let mut negative = negatives[0];
    let mut residuals = Vec::with_capacity(values.len());

    loop {
        let residual = positive + negative;
        if residual > 0.0 {
            positive = residual;
            negative_index += 1;
            if negative_index == negatives.len() {
                residuals.push(positive);
                residuals.extend_from_slice(&positives[positive_index + 1..]);
                break;
            }
            negative = negatives[negative_index];
        } else if residual < 0.0 {
            negative = residual;
            positive_index += 1;
            if positive_index == positives.len() {
                residuals.push(negative);
                residuals.extend_from_slice(&negatives[negative_index + 1..]);
                break;
            }
            positive = positives[positive_index];
        } else {
            positive_index += 1;
            negative_index += 1;
            if positive_index == positives.len() || negative_index == negatives.len() {
                residuals.extend_from_slice(&positives[positive_index..]);
                residuals.extend_from_slice(&negatives[negative_index..]);
                break;
            }
            positive = positives[positive_index];
            negative = negatives[negative_index];
        }
    }

    if residuals.is_empty() {
        return Ok(0.0);
    }
    same_sign_mean_over_total(&residuals, values.len())
}

fn standard_error_from_deviations(deviations: &[f64]) -> Result<f64, ValidationError> {
    let scale = deviations
        .iter()
        .map(|deviation| deviation.abs())
        .fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }

    let normalized_squares: Vec<_> = deviations
        .iter()
        .map(|deviation| {
            let normalized = *deviation / scale;
            normalized * normalized
        })
        .collect();
    let square_sum = deterministic_compensated_sum(normalized_squares);
    let sample_variance_scale = square_sum / (deviations.len() as f64 - 1.0);
    let normalized_standard_error = sample_variance_scale.sqrt() / (deviations.len() as f64).sqrt();
    let standard_error = scale * normalized_standard_error;
    if !standard_error.is_finite()
        || (standard_error == 0.0 && normalized_standard_error != 0.0)
    {
        Err(ValidationError::InvalidInput)
    } else if standard_error == 0.0 {
        Ok(0.0)
    } else {
        Ok(standard_error)
    }
}

fn scaled_standard_error(values: &[f64], mean: f64) -> Result<f64, ValidationError> {
    let direct_deviations: Option<Vec<f64>> = values
        .iter()
        .map(|value| {
            let deviation = *value - mean;
            deviation.is_finite().then_some(deviation)
        })
        .collect();
    if let Some(deviations) = direct_deviations {
        return standard_error_from_deviations(&deviations);
    }

    let outer_scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if outer_scale == 0.0 {
        return Ok(0.0);
    }
    let normalized_values: Vec<_> = values.iter().map(|value| *value / outer_scale).collect();
    let normalized_mean = scaled_compensated_mean(&normalized_values)?;
    let normalized_deviations: Vec<_> = normalized_values
        .iter()
        .map(|value| *value - normalized_mean)
        .collect();
    let normalized_standard_error = standard_error_from_deviations(&normalized_deviations)?;
    let standard_error = outer_scale * normalized_standard_error;
    if !standard_error.is_finite()
        || (standard_error == 0.0 && normalized_standard_error != 0.0)
    {
        Err(ValidationError::InvalidInput)
    } else if standard_error == 0.0 {
        Ok(0.0)
    } else {
        Ok(standard_error)
    }
}

/// Mean signed bias `mean(recovered − truth)`.
///
/// Mixed-sign residuals cancel at their represented magnitudes before any
/// scale reduction, so a tiny residual needed after extreme cancellation is
/// not divided into zero. The remaining one-sign mass is normalized by an exact
/// power-of-two scale and summed deterministically before the original recovery
/// denominator is applied. This keeps representable bias finite without
/// allowing raw-sum overflow or false zero from scale normalization.
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
/// reference as [`mean_bias`]. Squared deviations are accumulated only after
/// scaling by their largest magnitude, and the standard error is formed
/// directly without materializing an avoidably overflowing raw variance.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for invalid pairs, `n < 2`, an
/// unrepresentable signed residual or mean, or an unrepresentable final
/// standard error.
pub fn bias_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    if truth.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    let diffs = signed_residuals(truth, recovered)?;
    let mean = scaled_compensated_mean(&diffs)?;
    scaled_standard_error(&diffs, mean)
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
        assert_eq!(mean_bias(&[1.0], &[1.0]), Ok(0.0));
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
        assert_eq!(se_var, 1.0);
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
    fn representable_standard_error_avoids_square_and_variance_overflow() {
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[f64::MAX, -f64::MAX]),
            Ok(f64::MAX)
        );
        let huge = 1e200;
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[huge, -huge]),
            Ok(huge)
        );
        let square_sum_overflows = 1e154;
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[square_sum_overflows, -square_sum_overflows]),
            Ok(square_sum_overflows)
        );
        let three_point = bias_standard_error(
            &[0.0, 0.0, 0.0],
            &[square_sum_overflows, -square_sum_overflows, 0.0],
        )
        .expect("representable three-point standard error");
        let expected = square_sum_overflows / 3.0_f64.sqrt();
        assert!(((three_point - expected) / expected).abs() <= f64::EPSILON);
    }

    #[test]
    fn overflowing_direct_deviation_uses_scaled_reference() {
        let standard_error = bias_standard_error(
            &[0.0, 0.0, 0.0],
            &[f64::MAX, -f64::MAX, -f64::MAX],
        )
        .expect("scaled deviation path");
        assert!(standard_error.is_finite());
        assert!(standard_error > 0.0);
    }
}
