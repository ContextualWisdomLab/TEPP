//! Interval coverage for recovered confidence/credible intervals.

use crate::ValidationError;

pub(crate) fn interval_covered_count(
    truth: &[f64],
    lower: &[f64],
    upper: &[f64],
) -> Result<usize, ValidationError> {
    if truth.is_empty() || truth.len() != lower.len() || truth.len() != upper.len() {
        return Err(ValidationError::InvalidInput);
    }
    let mut covered = 0usize;
    for index in 0..truth.len() {
        let t = truth[index];
        let lo = lower[index];
        let hi = upper[index];
        if !t.is_finite() || !lo.is_finite() || !hi.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        if lo > hi {
            return Err(ValidationError::InvalidInput);
        }
        if t >= lo && t <= hi {
            covered += 1;
        }
    }
    Ok(covered)
}

fn correctly_rounded_unit_ratio(numerator: u64, denominator: u64) -> f64 {
    debug_assert!(denominator > 0);
    debug_assert!(numerator <= denominator);
    if numerator == 0 {
        return 0.0;
    }

    // u64/u64 lies in [2^-64, 1], so its binary64 result is always normal.
    // Determine floor(log2(numerator/denominator)) without first rounding either
    // integer to f64, then round the exact scaled significand ties-to-even.
    let numerator_bits = 64_i32 - numerator.leading_zeros() as i32;
    let denominator_bits = 64_i32 - denominator.leading_zeros() as i32;
    let mut exponent = numerator_bits - denominator_bits;
    if exponent == 0 {
        if numerator < denominator {
            exponent = -1;
        }
    } else {
        let exponent_shift = (-exponent) as u32;
        if (numerator as u128) << exponent_shift < denominator as u128 {
            exponent -= 1;
        }
    }

    let significand_shift = (52 - exponent) as u32;
    let scaled_numerator = (numerator as u128) << significand_shift;
    let denominator_u128 = denominator as u128;
    let quotient = scaled_numerator / denominator_u128;
    let remainder = scaled_numerator % denominator_u128;
    let twice_remainder = remainder << 1;
    let round_up = twice_remainder > denominator_u128
        || (twice_remainder == denominator_u128 && quotient & 1 == 1);
    let mut significand = quotient + u128::from(round_up);

    if significand == 1_u128 << 53 {
        significand >>= 1;
        exponent += 1;
    }

    debug_assert!((1_u128 << 52..1_u128 << 53).contains(&significand));
    let biased_exponent = (exponent + 1023) as u64;
    let fraction = significand as u64 - (1_u64 << 52);
    f64::from_bits((biased_exponent << 52) | fraction)
}

fn u64_is_exact_binary64_integer(value: u64) -> bool {
    if value == 0 {
        return true;
    }
    let significant_bits = 64 - value.leading_zeros();
    if significant_bits <= 53 {
        return true;
    }
    let discarded_bits = significant_bits - 53;
    let discarded_mask = (1_u64 << discarded_bits) - 1;
    value & discarded_mask == 0
}

pub(crate) fn represented_coverage_from_counts(
    covered_count: u64,
    sample_count: u64,
) -> Result<f64, ValidationError> {
    if sample_count == 0 || covered_count > sample_count {
        return Err(ValidationError::InvalidInput);
    }
    Ok(correctly_rounded_unit_ratio(covered_count, sample_count))
}

/// Empirical coverage of closed intervals `[lower, upper]` for truth values.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when vectors are empty, lengths
/// differ, bounds are non-finite, or any interval is inverted (`lower > upper`).
pub fn interval_coverage(
    truth: &[f64],
    lower: &[f64],
    upper: &[f64],
) -> Result<f64, ValidationError> {
    let covered = interval_covered_count(truth, lower, upper)?;
    represented_coverage_from_counts(covered as u64, truth.len() as u64)
}

fn rationalized_wilson_positive_lower(n: f64, p: f64, z: f64, z2: f64) -> f64 {
    if z2 >= 1.0 {
        // Rationalize the lower root and divide through by z²:
        //   2 n p² / (z² + 2 n p + z sqrt(z² + 4 n p (1-p))).
        // This form avoids subtracting nearly equal O(z²) terms and avoids
        // materializing the O(z²) denominator sum directly.
        let normalized_numerator = 2.0 * n * p * p / z2;
        let normalized_denominator = 1.0
            + 2.0 * n * p / z2
            + (1.0 + 4.0 * n * p * (1.0 - p) / z2).sqrt();
        return (normalized_numerator / normalized_denominator).clamp(0.0, 1.0);
    }

    // For z² < 1, dividing through by z² can overflow even though the Wilson
    // endpoint is ordinary and representable. Evaluate the same rationalized
    // root on its natural scale instead.
    let numerator = 2.0 * n * p * p;
    let denominator =
        z2 + 2.0 * n * p + z * (z2 + 4.0 * n * p * (1.0 - p)).sqrt();
    (numerator / denominator).clamp(0.0, 1.0)
}

fn rationalized_wilson_positive_lower_from_inverse_sample_count(
    inverse_n: f64,
    p: f64,
    z: f64,
    z2: f64,
) -> f64 {
    // This path is used only when the exact u64 denominator cannot be represented
    // as binary64. Such counts exceed 2^53, so inverse_n is small enough that
    // the natural reciprocal-scale form remains finite even for the largest z
    // whose square is finite. It avoids materializing a rounded sample count.
    let numerator = 2.0 * p * p;
    let inverse_n_squared = inverse_n * inverse_n;
    let denominator = z2 * inverse_n
        + 2.0 * p
        + z * (z2 * inverse_n_squared + 4.0 * p * (1.0 - p) * inverse_n).sqrt();
    (numerator / denominator).clamp(0.0, 1.0)
}

fn all_covered_wilson_lower_from_inverse_sample_count(inverse_n: f64, z2: f64) -> f64 {
    let z2_over_n = z2 * inverse_n;
    if z2_over_n <= 1.0 {
        // For small z²/n, 1 / (1 + z²/n) can round to exact one before the
        // finite-sample miss mass is represented. Subtract the miss mass instead.
        let uncovered_mass = z2_over_n / (1.0 + z2_over_n);
        (1.0 - uncovered_mass).clamp(0.0, 1.0)
    } else {
        // For large z²/n, the complementary miss mass rounds to exact one and
        // subtraction would erase an ordinary positive lower endpoint.
        (1.0 / (1.0 + z2_over_n)).clamp(0.0, 1.0)
    }
}

fn wilson_bounds_from_represented_proportion(
    n: f64,
    p: f64,
    z: f64,
    z2: f64,
) -> (f64, f64) {
    let low = if p > 0.0 {
        rationalized_wilson_positive_lower(n, p, z, z2)
    } else {
        0.0
    };

    let denominator = 1.0 + z2 / n;
    let center = p + z2 / (2.0 * n);
    let radical = (p * (1.0 - p) / n) + z2 / (4.0 * n * n);
    let margin = z * radical.sqrt();
    let direct_high = ((center + margin) / denominator).clamp(0.0, 1.0);
    let high = if direct_high == 1.0 && p < 1.0 && z2 > 0.0 {
        let uncovered_lower = rationalized_wilson_positive_lower(n, 1.0 - p, z, z2);
        (1.0 - uncovered_lower).clamp(0.0, 1.0)
    } else {
        direct_high
    };
    (low, high)
}

fn wilson_bounds_from_represented_proportion_and_inverse_sample_count(
    inverse_n: f64,
    p: f64,
    z: f64,
    z2: f64,
) -> (f64, f64) {
    let low = if p > 0.0 {
        rationalized_wilson_positive_lower_from_inverse_sample_count(inverse_n, p, z, z2)
    } else {
        0.0
    };

    let z2_over_n = z2 * inverse_n;
    let denominator = 1.0 + z2_over_n;
    let center = p + z2_over_n / 2.0;
    let radical = p * (1.0 - p) * inverse_n + z2 * inverse_n * inverse_n / 4.0;
    let margin = z * radical.sqrt();
    let direct_high = ((center + margin) / denominator).clamp(0.0, 1.0);
    let high = if direct_high == 1.0 && p < 1.0 && z2 > 0.0 {
        let uncovered_lower = rationalized_wilson_positive_lower_from_inverse_sample_count(
            inverse_n,
            1.0 - p,
            z,
            z2,
        );
        (1.0 - uncovered_lower).clamp(0.0, 1.0)
    } else {
        direct_high
    };
    (low, high)
}

fn wilson_bounds_for_sample_count(
    n: f64,
    inverse_n: Option<f64>,
    p: f64,
    z: f64,
    z2: f64,
) -> (f64, f64) {
    if let Some(inverse_n) = inverse_n {
        wilson_bounds_from_represented_proportion_and_inverse_sample_count(inverse_n, p, z, z2)
    } else {
        wilson_bounds_from_represented_proportion(n, p, z, z2)
    }
}

pub(crate) fn wilson_coverage_interval_from_counts(
    covered_count: u64,
    sample_count: u64,
    z: f64,
) -> Result<(f64, f64), ValidationError> {
    if sample_count == 0 || covered_count > sample_count {
        return Err(ValidationError::InvalidInput);
    }
    if !z.is_finite() || z <= 0.0 {
        return Err(ValidationError::InvalidConfiguration);
    }

    let n = sample_count as f64;
    let z2 = z * z;
    if !z2.is_finite() {
        return Err(ValidationError::InvalidConfiguration);
    }
    let inverse_n = if u64_is_exact_binary64_integer(sample_count) {
        None
    } else {
        Some(correctly_rounded_unit_ratio(1, sample_count))
    };

    if covered_count == sample_count {
        if let Some(inverse_n) = inverse_n {
            return Ok((
                all_covered_wilson_lower_from_inverse_sample_count(inverse_n, z2),
                1.0,
            ));
        }
        return Ok((n / (n + z2), 1.0));
    }

    let uncovered_count = sample_count - covered_count;
    if covered_count > uncovered_count {
        // Near all-covered samples can lose observed misses if both integer
        // counts are independently rounded before division. Wilson intervals
        // are complement-symmetric, so evaluate the correctly rounded smaller
        // uncovered proportion and reflect its endpoints instead.
        let uncovered = correctly_rounded_unit_ratio(uncovered_count, sample_count);
        let (uncovered_low, uncovered_high) =
            wilson_bounds_for_sample_count(n, inverse_n, uncovered, z, z2);
        return Ok((
            (1.0 - uncovered_high).clamp(0.0, 1.0),
            (1.0 - uncovered_low).clamp(0.0, 1.0),
        ));
    }

    let p = correctly_rounded_unit_ratio(covered_count, sample_count);
    Ok(wilson_bounds_for_sample_count(n, inverse_n, p, z, z2))
}

/// Wilson score lower/upper bounds for a binomial coverage proportion.
///
/// Returns `(lower, upper)` for the empirical coverage rate at the stated
/// normal critical value `z` (for example `1.96` for nominal 95%). For an
/// all-covered sample, the exact Wilson lower endpoint is evaluated as
/// `n / (n + z²)`. For nonzero strict-interior coverage, the lower endpoint is
/// evaluated through the algebraically rationalized positive root rather than
/// `center - margin`; the implementation switches scale at `z² = 1` so the
/// stable form neither suffers large-z cancellation nor small-z division
/// overflow. Count proportions are rounded to binary64 from their exact integer
/// ratio before Wilson evaluation. When the exact sample count itself is not
/// binary64-representable, Wilson scale terms are evaluated through the
/// correctly rounded reciprocal `1 / n` rather than a pre-rounded `n as f64`;
/// the all-covered branch switches between complementary-miss and direct
/// reciprocal forms at `z² / n = 1`, preserving both tiny uncertainty below one
/// and positive lower endpoints under extreme finite critical values. Near the
/// all-covered boundary, the smaller uncovered count is evaluated and reflected
/// by Wilson complement symmetry.
///
/// # Errors
///
/// Returns configuration errors for non-finite `z` or `z <= 0`, and input
/// errors for empty/invalid interval triples.
pub fn wilson_coverage_interval(
    truth: &[f64],
    lower: &[f64],
    upper: &[f64],
    z: f64,
) -> Result<(f64, f64), ValidationError> {
    if !z.is_finite() || z <= 0.0 {
        return Err(ValidationError::InvalidConfiguration);
    }
    let covered_count = interval_covered_count(truth, lower, upper)?;
    wilson_coverage_interval_from_counts(covered_count as u64, truth.len() as u64, z)
}

#[cfg(test)]
mod tests {
    use super::{
        correctly_rounded_unit_ratio, interval_coverage, u64_is_exact_binary64_integer,
        wilson_coverage_interval,
    };
    use crate::ValidationError;

    #[test]
    fn exact_integer_ratio_rounding_covers_binary64_boundaries() {
        assert_eq!(correctly_rounded_unit_ratio(0, 3), 0.0);
        assert_eq!(correctly_rounded_unit_ratio(1, 1), 1.0);
        assert_eq!(correctly_rounded_unit_ratio(1, 2), 0.5);
        assert_eq!(correctly_rounded_unit_ratio(1, 3), 1.0 / 3.0);

        // Halfway between 0.5 and its successor: lower significand is even.
        assert_eq!(
            correctly_rounded_unit_ratio((1_u64 << 53) + 1, 1_u64 << 54),
            0.5
        );
        // Just above that midpoint rounds upward.
        assert_eq!(
            correctly_rounded_unit_ratio((1_u64 << 54) + 3, 1_u64 << 55).to_bits(),
            0.5_f64.to_bits() + 1
        );
        // Just below the midpoint rounds downward.
        assert_eq!(
            correctly_rounded_unit_ratio((1_u64 << 54) + 1, 1_u64 << 55),
            0.5
        );
        // Halfway between predecessor(1.0) and 1.0: 1.0 has the even
        // significand, so ties-to-even rounds upward and renormalizes.
        assert_eq!(
            correctly_rounded_unit_ratio((1_u64 << 54) - 1, 1_u64 << 54),
            1.0
        );
    }

    #[test]
    fn sample_count_exactness_detection_matches_binary64_integer_spacing() {
        assert!(u64_is_exact_binary64_integer(0));
        assert!(u64_is_exact_binary64_integer(1_u64 << 53));
        assert!(!u64_is_exact_binary64_integer((1_u64 << 53) + 1));
        assert!(u64_is_exact_binary64_integer((1_u64 << 53) + 2));
        assert!(!u64_is_exact_binary64_integer((1_u64 << 55) + 3));
        assert!(u64_is_exact_binary64_integer((1_u64 << 55) + 8));
    }

    #[test]
    fn coverage_and_wilson_bounds_are_oracle_correct() {
        let truth = [0.0, 1.0, 2.0, 3.0];
        let lower = [-0.5, 0.5, 1.5, 4.0];
        let upper = [0.5, 1.5, 2.5, 5.0];
        // first three covered, last not → 0.75
        assert!((interval_coverage(&truth, &lower, &upper).expect("cov") - 0.75).abs() < 1e-12);
        let (lo, hi) = wilson_coverage_interval(&truth, &lower, &upper, 1.96).expect("wilson");
        assert!(lo <= 0.75);
        assert!(0.75 <= hi);
        assert_eq!(
            interval_coverage(&[], &[], &[]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[1.0], &[2.0], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[1.0], &[0.0, 1.0], &[2.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[1.0], &[0.0], &[2.0, 3.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[f64::NAN], &[0.0], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[0.5], &[f64::NAN], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[0.5], &[0.0], &[f64::INFINITY]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, 0.0),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, -1.0),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, f64::NAN),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, f64::MAX),
            Err(ValidationError::InvalidConfiguration)
        );
        // Finite z whose scaled Wilson terms still overflow.
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, 1e200),
            Err(ValidationError::InvalidConfiguration)
        );
        // uncovered: above interval and below interval
        let miss_high = interval_coverage(&[0.0], &[-2.0], &[-1.0]).expect("miss high");
        assert!((miss_high - 0.0).abs() < 1e-12);
        let miss_low = interval_coverage(&[0.0], &[1.0], &[2.0]).expect("miss low");
        assert!((miss_low - 0.0).abs() < 1e-12);
    }

    #[test]
    fn wilson_nonfinite_guards() {
        let truth = [0.0];
        let lower = [-1.0];
        let upper = [1.0];
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, f64::MAX),
            Err(ValidationError::InvalidConfiguration)
        );
        // Finite z whose scaled Wilson terms still overflow.
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, 1e200),
            Err(ValidationError::InvalidConfiguration)
        );
    }
}
