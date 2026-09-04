//! Deterministic binary64 accumulation shared by validation metrics.

use crate::ValidationError;

/// Sum finite values in a canonical order with Neumaier compensation.
///
/// Callers own domain validation and any scale normalization needed to keep the
/// final sum representable. Canonical ordering keeps equivalent metric inputs
/// from changing only because transport order changed.
pub(crate) fn deterministic_compensated_sum(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for value in values {
        let next = sum + value;
        if sum.abs() >= value.abs() {
            correction += (sum - next) + value;
        } else {
            correction += (value - next) + sum;
        }
        sum = next;
    }
    sum + correction
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
        .fold(0.0, f64::max);
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

fn error_free_sum(left: f64, right: f64) -> (f64, f64) {
    let sum = left + right;
    let right_virtual = sum - left;
    let left_virtual = sum - right_virtual;
    let left_roundoff = left - left_virtual;
    let right_roundoff = right - right_virtual;
    (sum, left_roundoff + right_roundoff)
}

fn mixed_remainder_mean_over_total(
    values: &[f64],
    total_count: usize,
) -> Result<f64, ValidationError> {
    let max_magnitude = values
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    if max_magnitude == 0.0 {
        return Ok(0.0);
    }

    let scale = exact_power_of_two_scale(max_magnitude);
    let normalized = values.iter().map(|value| *value / scale).collect();
    let normalized_sum = deterministic_compensated_sum(normalized);
    if normalized_sum == 0.0 {
        return Ok(0.0);
    }

    let normalized_mean = normalized_sum / total_count as f64;
    let mean = normalized_mean * scale;
    if !mean.is_finite() || mean == 0.0 {
        Err(ValidationError::InvalidInput)
    } else {
        Ok(mean)
    }
}

/// Deterministically divide the represented sum of finite binary64 values by an explicit count.
///
/// Opposite signs cancel before exact power-of-two scale reduction. Each
/// opposite-sign addition also retains its error-free low term so repeated
/// sub-ULP contributions cannot disappear one at a time before they collectively
/// become representable. The divisor is independent of `values.len()`, which
/// lets callers preserve an original scientific denominator when an algebraically
/// equivalent expanded term set is needed to avoid overflowing intermediate
/// differences. Exact cancellation returns canonical zero; a nonzero quotient
/// outside or below binary64 range fails closed.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for an empty value set, a zero
/// divisor, non-finite input, or an unrepresentable nonzero quotient.
pub(crate) fn deterministic_representable_sum_over_count(
    values: &[f64],
    total_count: usize,
) -> Result<f64, ValidationError> {
    if values.is_empty() || total_count == 0 || values.iter().any(|value| !value.is_finite()) {
        return Err(ValidationError::InvalidInput);
    }

    let mut positives = Vec::new();
    let mut negatives = Vec::new();
    for &value in values {
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
        return same_sign_mean_over_total(values, total_count);
    }

    positives.sort_by(|left, right| right.total_cmp(left));
    negatives.sort_by(|left, right| left.total_cmp(right));

    let mut positive_index = 0_usize;
    let mut negative_index = 0_usize;
    let mut positive = positives[0];
    let mut negative = negatives[0];
    let mut residuals = Vec::with_capacity(values.len());
    let mut roundoff_terms = Vec::new();

    loop {
        let (residual, roundoff) = error_free_sum(positive, negative);
        if roundoff != 0.0 {
            roundoff_terms.push(roundoff);
        }

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

    if roundoff_terms.is_empty() {
        if residuals.is_empty() {
            return Ok(0.0);
        }
        return same_sign_mean_over_total(&residuals, total_count);
    }

    residuals.extend(roundoff_terms);
    mixed_remainder_mean_over_total(&residuals, total_count)
}

/// Deterministic mean of finite binary64 values with cancellation before scale reduction.
///
/// Opposite signs cancel at represented magnitude before the remaining mass is
/// normalized by an exact power of two. Error-free low terms from cancellation
/// are retained so several individually sub-ULP contributions can still affect
/// the represented mean when their combined mass is large enough. The original
/// sample count stays in the denominator after cancellation. Exact all-zero input
/// and exact mixed-sign cancellation return canonical zero; a mathematically
/// nonzero mean that falls below binary64 range fails closed.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty or non-finite input, or
/// when a nonzero represented mass has no nonzero binary64 mean.
pub(crate) fn deterministic_representable_mean(values: &[f64]) -> Result<f64, ValidationError> {
    deterministic_representable_sum_over_count(values, values.len())
}

#[cfg(test)]
mod tests {
    use super::{
        deterministic_compensated_sum, deterministic_representable_mean,
        deterministic_representable_sum_over_count,
    };
    use crate::ValidationError;

    #[test]
    fn canonical_compensated_sum_is_order_stable() {
        let left = deterministic_compensated_sum(vec![1.0, 1e-100, -1.0]);
        let right = deterministic_compensated_sum(vec![-1.0, 1.0, 1e-100]);
        assert_eq!(left.to_bits(), right.to_bits());
    }

    #[test]
    fn representable_mean_preserves_full_range_cancellation() {
        let minimum_subnormal = f64::from_bits(1);
        let twice_minimum_subnormal = f64::from_bits(2);
        let positive = [
            f64::MAX,
            twice_minimum_subnormal,
            twice_minimum_subnormal,
            -f64::MAX,
        ];
        let negative = [
            -f64::MAX,
            -twice_minimum_subnormal,
            -twice_minimum_subnormal,
            f64::MAX,
        ];
        assert_eq!(
            deterministic_representable_mean(&positive)
                .expect("positive")
                .to_bits(),
            minimum_subnormal.to_bits()
        );
        assert_eq!(
            deterministic_representable_mean(&negative)
                .expect("negative")
                .to_bits(),
            (-minimum_subnormal).to_bits()
        );
        assert_eq!(
            deterministic_representable_mean(&[f64::MAX, -f64::MAX]),
            Ok(0.0)
        );
    }

    #[test]
    fn representable_mean_retains_accumulated_opposite_sign_roundoff() {
        let quarter_ulp_at_one = 2.0_f64.powi(-54);
        let mean = deterministic_representable_mean(&[
            1.0,
            -quarter_ulp_at_one,
            -quarter_ulp_at_one,
            -quarter_ulp_at_one,
            -quarter_ulp_at_one,
        ])
        .expect("representable mixed-sign mean");
        assert_eq!(mean.to_bits(), 0x3fc9_9999_9999_9998);
    }

    #[test]
    fn explicit_denominator_preserves_representable_expanded_sum() {
        let minimum_subnormal = f64::from_bits(1);
        assert_eq!(
            deterministic_representable_sum_over_count(
                &[f64::MAX, -f64::MAX, f64::from_bits(3)],
                3,
            )
            .expect("explicit denominator")
            .to_bits(),
            minimum_subnormal.to_bits()
        );
        assert_eq!(
            deterministic_representable_sum_over_count(&[f64::MAX, f64::MAX], 1),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            deterministic_representable_sum_over_count(&[0.0], 0),
            Err(ValidationError::InvalidInput)
        );
    }

    #[test]
    fn representable_mean_covers_admission_and_residual_paths() {
        let minimum_subnormal = f64::from_bits(1);
        assert_eq!(
            deterministic_representable_mean(&[]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            deterministic_representable_mean(&[f64::NAN]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(deterministic_representable_mean(&[0.0, -0.0]), Ok(0.0));
        assert_eq!(deterministic_representable_mean(&[1.0, 1.0]), Ok(1.0));
        assert_eq!(deterministic_representable_mean(&[-1.0, -1.0]), Ok(-1.0));
        assert_eq!(
            deterministic_representable_mean(&[minimum_subnormal, 0.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            deterministic_representable_mean(&[3.0, -1.0, -1.0]),
            Ok(1.0 / 3.0)
        );
        assert_eq!(
            deterministic_representable_mean(&[-3.0, 1.0, 1.0]),
            Ok(-1.0 / 3.0)
        );
        assert_eq!(
            deterministic_representable_mean(&[3.0, -1.0, -1.0, -1.0]),
            Ok(0.0)
        );
    }
}
