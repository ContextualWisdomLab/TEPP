//! Interval coverage for recovered confidence/credible intervals.

use crate::ValidationError;

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
        let low_ok = t >= lo;
        let high_ok = t <= hi;
        if low_ok && high_ok {
            covered += 1;
        }
    }
    Ok(covered as f64 / truth.len() as f64)
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

/// Wilson score lower/upper bounds for a binomial coverage proportion.
///
/// Returns `(lower, upper)` for the empirical coverage rate at the stated
/// normal critical value `z` (for example `1.96` for nominal 95%). For an
/// all-covered sample, the exact Wilson lower endpoint is evaluated as
/// `n / (n + z²)`. For nonzero strict-interior coverage, the lower endpoint is
/// evaluated through the algebraically rationalized positive root rather than
/// `center - margin`; the implementation switches scale at `z² = 1` so the
/// stable form neither suffers large-z cancellation nor small-z division
/// overflow. The same positive-lower representation is applied to the
/// complementary uncovered proportion when `center + margin` falsely rounds an
/// upper endpoint to exact one even though the represented Wilson endpoint
/// remains below one.
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
    let p = interval_coverage(truth, lower, upper)?;
    let n = truth.len() as f64;
    let z2 = z * z;
    if !z2.is_finite() {
        return Err(ValidationError::InvalidConfiguration);
    }
    if p == 1.0 {
        return Ok((n / (n + z2), 1.0));
    }

    let low = if p > 0.0 {
        rationalized_wilson_positive_lower(n, p, z, z2)
    } else {
        0.0
    };

    let denominator = 1.0 + z2 / n;
    let center = p + z2 / (2.0 * n);
    let radical = (p * (1.0 - p) / n) + z2 / (4.0 * n * n);
    // With finite z² and coverage p in [0,1], Wilson terms remain finite.
    let margin = z * radical.sqrt();
    let direct_high = ((center + margin) / denominator).clamp(0.0, 1.0);
    let high = if direct_high == 1.0 && p < 1.0 && z2 > 0.0 {
        let uncovered_lower = rationalized_wilson_positive_lower(n, 1.0 - p, z, z2);
        (1.0 - uncovered_lower).clamp(0.0, 1.0)
    } else {
        direct_high
    };
    Ok((low, high))
}

#[cfg(test)]
mod tests {
    use super::{interval_coverage, wilson_coverage_interval};
    use crate::ValidationError;

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
