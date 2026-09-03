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
    if mean == 0.0 {
        Err(ValidationError::InvalidInput)
    } else {
        Ok(mean)
    }
}

/// Deterministic mean of finite binary64 values with cancellation before scale reduction.
///
/// Opposite signs cancel at represented magnitude before the remaining one-sign
/// mass is normalized by an exact power of two. The original sample count stays
/// in the denominator after cancellation. Exact all-zero input and exact mixed-
/// sign cancellation return canonical zero; a mathematically nonzero one-sign
/// mean that falls below binary64 range fails closed.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty or non-finite input, or
/// when a nonzero represented mass has no nonzero binary64 mean.
pub(crate) fn deterministic_representable_mean(values: &[f64]) -> Result<f64, ValidationError> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
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

#[cfg(test)]
mod tests {
    use super::{deterministic_compensated_sum, deterministic_representable_mean};
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
