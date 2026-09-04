//! Monte Carlo aggregation of recovery metrics.

use crate::ValidationError;
use crate::input::require_finite;
use crate::numeric::{deterministic_compensated_sum, deterministic_representable_mean};

const STANDARD_ERROR_RELATIVE_TOLERANCE: f64 = 64.0 * f64::EPSILON;
const EMPIRICAL_SUPPORT_RELATIVE_TOLERANCE: f64 = 64.0 * f64::EPSILON;

/// Summary of Monte Carlo replications for a scalar metric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonteCarloSummary {
    /// Number of finite replications retained.
    pub replication_count: usize,
    /// Sample mean.
    pub mean: f64,
    /// Sample standard deviation (`n − 1` denominator).
    pub standard_deviation: f64,
    /// Standard error of the mean.
    pub standard_error: f64,
    /// Inclusive empirical percentile lower bound.
    pub percentile_lower: f64,
    /// Inclusive empirical percentile upper bound.
    pub percentile_upper: f64,
}

impl MonteCarloSummary {
    /// Validate structural and uncertainty-domain invariants for a Monte Carlo summary payload.
    ///
    /// `standard_error` is the standard error of the retained-replication mean,
    /// so a positive sample SD must agree numerically with `SD / sqrt(n)`.
    /// Admission allows a small relative binary64 tolerance rather than requiring
    /// cross-language bit-for-bit equality, but rejects materially understated or
    /// overstated uncertainty. Zero spread requires zero SE and degenerate
    /// empirical percentile support at the represented mean; the same support
    /// rule applies to the canonical singleton summary. For positive spread,
    /// nearest-rank percentile endpoints are retained observations and therefore
    /// must fit the support implied directly by the recorded sample spread:
    /// `|x - mean| <= SD * sqrt(n - 1)`. Distinct lower and upper endpoints are
    /// distinct retained values, so their squared deviations must also fit the
    /// same total `(n - 1) * SD^2` deviation budget jointly. With exactly two
    /// replications, two distinct nearest-rank endpoint values exhaust the sample;
    /// the recorded mean and sample SD must therefore agree with the summary of
    /// those two endpoint values themselves. These rules remain valid when the
    /// represented binary64 mean is a rounded projection of the mathematical
    /// sample mean. Comparisons are scale-normalized so opposite-sign full-range
    /// finite values do not create overflowing validation-only arithmetic.
    /// Numeric equality keeps IEEE `-0.0` and `+0.0` as one zero-valued scientific state.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when counts or numeric fields
    /// violate the summary contract, empirical percentile support is impossible
    /// for the represented mean/sample spread/count, or the standard-error field
    /// is incoherent with the represented sample spread/count.
    pub fn validate(self) -> Result<Self, ValidationError> {
        if self.replication_count == 0 {
            return Err(ValidationError::InvalidInput);
        }
        for value in [
            self.mean,
            self.standard_deviation,
            self.standard_error,
            self.percentile_lower,
            self.percentile_upper,
        ] {
            if !value.is_finite() {
                return Err(ValidationError::InvalidInput);
            }
        }
        if self.standard_deviation < 0.0 || self.standard_error < 0.0 {
            return Err(ValidationError::InvalidInput);
        }
        if self.percentile_lower > self.percentile_upper {
            return Err(ValidationError::InvalidInput);
        }
        if self.standard_deviation == 0.0 {
            if self.standard_error != 0.0
                || self.percentile_lower != self.mean
                || self.percentile_upper != self.mean
            {
                return Err(ValidationError::InvalidInput);
            }
        } else {
            if self.replication_count == 1 || self.standard_error == 0.0 {
                return Err(ValidationError::InvalidInput);
            }
            let expected_standard_error =
                self.standard_deviation / (self.replication_count as f64).sqrt();
            if expected_standard_error == 0.0 {
                return Err(ValidationError::InvalidInput);
            }
            let relative_error =
                (self.standard_error / expected_standard_error - 1.0).abs();
            if !relative_error.is_finite() || relative_error > STANDARD_ERROR_RELATIVE_TOLERANCE {
                return Err(ValidationError::InvalidInput);
            }

            let moment_factor = ((self.replication_count - 1) as f64).sqrt();
            for endpoint in [self.percentile_lower, self.percentile_upper] {
                let scale = self
                    .mean
                    .abs()
                    .max(endpoint.abs())
                    .max(self.standard_deviation)
                    .max(1.0);
                let scaled_deviation = ((endpoint / scale) - (self.mean / scale)).abs();
                let scaled_support = (self.standard_deviation / scale) * moment_factor;
                if !scaled_deviation.is_finite()
                    || !scaled_support.is_finite()
                    || scaled_deviation
                        > scaled_support * (1.0 + EMPIRICAL_SUPPORT_RELATIVE_TOLERANCE)
                {
                    return Err(ValidationError::InvalidInput);
                }
            }

            if self.percentile_lower != self.percentile_upper {
                let scale = self
                    .mean
                    .abs()
                    .max(self.percentile_lower.abs())
                    .max(self.percentile_upper.abs())
                    .max(self.standard_deviation);
                let scaled_mean = self.mean / scale;
                let scaled_lower_deviation = (self.percentile_lower / scale) - scaled_mean;
                let scaled_upper_deviation = (self.percentile_upper / scale) - scaled_mean;
                let combined_scaled_deviation =
                    scaled_lower_deviation.hypot(scaled_upper_deviation);
                let scaled_support = (self.standard_deviation / scale) * moment_factor;
                if !combined_scaled_deviation.is_finite()
                    || !scaled_support.is_finite()
                    || combined_scaled_deviation
                        > scaled_support * (1.0 + EMPIRICAL_SUPPORT_RELATIVE_TOLERANCE)
                {
                    return Err(ValidationError::InvalidInput);
                }

                if self.replication_count == 2 {
                    let endpoint_samples = [self.percentile_lower, self.percentile_upper];
                    let expected_mean = deterministic_representable_mean(&endpoint_samples)?;
                    let expected_standard_deviation =
                        scaled_sample_standard_deviation(&endpoint_samples, expected_mean)?;
                    for (recorded, expected) in [
                        (self.mean, expected_mean),
                        (self.standard_deviation, expected_standard_deviation),
                    ] {
                        let coherence_scale = recorded.abs().max(expected.abs());
                        if coherence_scale == 0.0 {
                            continue;
                        }
                        let relative_distance =
                            ((recorded / coherence_scale) - (expected / coherence_scale)).abs();
                        if relative_distance > EMPIRICAL_SUPPORT_RELATIVE_TOLERANCE {
                            return Err(ValidationError::InvalidInput);
                        }
                    }
                }
            }
        }
        Ok(self)
    }
}

fn standard_deviation_from_deviations(deviations: &[f64]) -> Result<f64, ValidationError> {
    let scale = deviations
        .iter()
        .map(|deviation| deviation.abs())
        .fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }

    let normalized_squares = deviations
        .iter()
        .map(|deviation| {
            let normalized = *deviation / scale;
            normalized * normalized
        })
        .collect();
    let square_sum = deterministic_compensated_sum(normalized_squares);
    let normalized_standard_deviation =
        (square_sum / (deviations.len() as f64 - 1.0)).sqrt();
    let standard_deviation = scale * normalized_standard_deviation;
    if !standard_deviation.is_finite()
        || (standard_deviation == 0.0 && normalized_standard_deviation != 0.0)
    {
        Err(ValidationError::InvalidInput)
    } else if standard_deviation == 0.0 {
        Ok(0.0)
    } else {
        Ok(standard_deviation)
    }
}

fn scaled_sample_standard_deviation(samples: &[f64], mean: f64) -> Result<f64, ValidationError> {
    let direct_deviations: Option<Vec<f64>> = samples
        .iter()
        .map(|value| {
            let deviation = *value - mean;
            deviation.is_finite().then_some(deviation)
        })
        .collect();
    if let Some(deviations) = direct_deviations {
        return standard_deviation_from_deviations(&deviations);
    }

    let outer_scale = samples.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if outer_scale == 0.0 {
        return Ok(0.0);
    }
    let normalized_mean = mean / outer_scale;
    let normalized_deviations: Vec<_> = samples
        .iter()
        .map(|value| (*value / outer_scale) - normalized_mean)
        .collect();
    let normalized_standard_deviation =
        standard_deviation_from_deviations(&normalized_deviations)?;
    let standard_deviation = outer_scale * normalized_standard_deviation;
    if !standard_deviation.is_finite()
        || (standard_deviation == 0.0 && normalized_standard_deviation != 0.0)
    {
        Err(ValidationError::InvalidInput)
    } else if standard_deviation == 0.0 {
        Ok(0.0)
    } else {
        Ok(standard_deviation)
    }
}

/// Aggregate Monte Carlo metric replications with percentile bounds.
///
/// Percentiles use the inclusive nearest-rank method on sorted finite samples.
/// Mean and sampling uncertainty use deterministic cancellation-safe/scaled
/// binary64 references so an avoidable raw sum, Welford delta product, or raw
/// square cannot reject a representable summary. A mathematically nonzero
/// standard deviation or standard error that becomes exact zero only at the
/// binary64 projection boundary fails closed rather than reporting no Monte
/// Carlo uncertainty.
///
/// # Errors
///
/// Returns input errors for empty/non-finite samples, an unrepresentable mean,
/// standard deviation, standard error, or summary; and configuration errors for
/// invalid percentile bounds.
///
/// # Panics
///
/// Does not panic: samples are pre-validated as finite before sorting.
pub fn summarize_replications(
    samples: &[f64],
    lower_percentile: f64,
    upper_percentile: f64,
) -> Result<MonteCarloSummary, ValidationError> {
    if samples.is_empty() || samples.iter().any(|value| !value.is_finite()) {
        return Err(ValidationError::InvalidInput);
    }
    if !(0.0..=1.0).contains(&lower_percentile)
        || !(0.0..=1.0).contains(&upper_percentile)
        || lower_percentile > upper_percentile
    {
        return Err(ValidationError::InvalidConfiguration);
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mean = deterministic_representable_mean(&sorted)?;
    let n = sorted.len() as f64;
    let standard_deviation = if sorted.len() == 1 {
        0.0
    } else {
        scaled_sample_standard_deviation(&sorted, mean)?
    };
    let standard_error = require_finite(standard_deviation / n.sqrt())?;
    if standard_error == 0.0 && standard_deviation != 0.0 {
        return Err(ValidationError::InvalidInput);
    }
    let summary = MonteCarloSummary {
        replication_count: sorted.len(),
        mean,
        standard_deviation,
        standard_error,
        percentile_lower: nearest_rank(&sorted, lower_percentile),
        percentile_upper: nearest_rank(&sorted, upper_percentile),
    };
    summary.validate()
}

/// Decode a nonnegative finite binary64 magnitude into an exact integer significand and power-of-two exponent.
fn binary64_magnitude_components(value: f64) -> (u64, i32) {
    let bits = value.to_bits() & 0x7fff_ffff_ffff_ffff;
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    if exponent_bits == 0 {
        (fraction, -1074)
    } else {
        ((1_u64 << 52) | fraction, exponent_bits - 1023 - 52)
    }
}

/// Compare two exact nonzero `significand * 2^exponent` values without floating-point rounding.
fn scaled_u128_le(
    lhs_significand: u128,
    lhs_exponent: i32,
    rhs_significand: u128,
    rhs_exponent: i32,
) -> bool {
    let lhs_bits = 128_i32 - lhs_significand.leading_zeros() as i32;
    let rhs_bits = 128_i32 - rhs_significand.leading_zeros() as i32;
    let lhs_top_exponent = lhs_exponent + lhs_bits - 1;
    let rhs_top_exponent = rhs_exponent + rhs_bits - 1;
    if lhs_top_exponent != rhs_top_exponent {
        return lhs_top_exponent < rhs_top_exponent;
    }

    let common_exponent = lhs_exponent.min(rhs_exponent);
    let lhs_shift = (lhs_exponent - common_exponent) as u32;
    let rhs_shift = (rhs_exponent - common_exponent) as u32;
    debug_assert!(lhs_shift < 128);
    debug_assert!(rhs_shift < 128);
    (lhs_significand << lhs_shift) <= (rhs_significand << rhs_shift)
}

/// Compare one represented nonnegative binary64 magnitude with an exact product of two represented factors.
fn represented_magnitude_le_exact_product(value: f64, factor_a: f64, factor_b: f64) -> bool {
    let (value_significand, value_exponent) = binary64_magnitude_components(value);
    let (a_significand, a_exponent) = binary64_magnitude_components(factor_a);
    let (b_significand, b_exponent) = binary64_magnitude_components(factor_b);
    scaled_u128_le(
        value_significand as u128,
        value_exponent,
        (a_significand as u128) * (b_significand as u128),
        a_exponent + b_exponent,
    )
}

/// Compare the exact represented residual magnitude with `k * SE` after both direct operations overflow.
fn both_overflow_acceptance(
    estimate: f64,
    target: f64,
    standard_error: f64,
    k: f64,
) -> bool {
    let (estimate_significand, estimate_exponent) =
        binary64_magnitude_components(estimate.abs());
    let (target_significand, target_exponent) = binary64_magnitude_components(target.abs());
    let common_residual_exponent = estimate_exponent.min(target_exponent);
    let estimate_shift = (estimate_exponent - common_residual_exponent) as u32;
    let target_shift = (target_exponent - common_residual_exponent) as u32;

    // Finite subtraction can overflow only for opposite signs whose magnitudes
    // are close enough to the top of binary64 that exact alignment needs at most
    // one 53-bit significand-width shift.
    debug_assert!(estimate_shift <= 53);
    debug_assert!(target_shift <= 53);
    let residual_significand = ((estimate_significand as u128) << estimate_shift)
        + ((target_significand as u128) << target_shift);

    let (k_significand, k_exponent) = binary64_magnitude_components(k);
    let (se_significand, se_exponent) = binary64_magnitude_components(standard_error);
    let bound_significand = (k_significand as u128) * (se_significand as u128);
    let bound_exponent = k_exponent + se_exponent;

    scaled_u128_le(
        residual_significand,
        common_residual_exponent,
        bound_significand,
        bound_exponent,
    )
}

/// Return the error-free low term of a finite binary64 subtraction.
fn subtraction_roundoff(minuend: f64, subtrahend: f64, difference: f64) -> f64 {
    let virtual_subtrahend = minuend - difference;
    let virtual_minuend = difference + virtual_subtrahend;
    let subtrahend_roundoff = virtual_subtrahend - subtrahend;
    let minuend_roundoff = minuend - virtual_minuend;
    minuend_roundoff + subtrahend_roundoff
}

/// SE-aware acceptance: accept when `|estimate − target| ≤ k · se`.
///
/// Finite represented residuals and finite represented `k · se` bounds are
/// compared before any normalization. If both rounded finite quantities are equal
/// and nonzero, TEPP compares the error-free subtraction correction (with its sign
/// adjusted for the absolute residual) with the fused multiply-add product
/// correction. Different correction projections preserve the represented-input
/// ordering even when either direct operation rounded to the same binary64 value.
/// When both projected corrections are zero at a subnormal rounded bound and the
/// subtraction was exact, TEPP compares that represented residual with the exact
/// dyadic product of the represented `k` and `se`; this prevents FMA-underflowed
/// product error from turning a strict rejection into equality. Other equal
/// correction projections remain on the ordinary rounded decision instead of
/// claiming a broader exact comparator. This also preserves a positive acceptance
/// bound when dividing `se` by a much larger estimate/target scale would underflow
/// to zero. If only the positive bound overflows, every finite residual is covered;
/// if only the residual overflows, a finite bound cannot cover it. When both direct
/// operations overflow, TEPP compares the exact binary64 input rationals by
/// decoding their integer significands and powers of two, avoiding a false
/// accept/reject caused by independently rounded normalization. A zero standard
/// error or zero multiplier remains an exact-recovery gate and is compared before
/// either path. Exact recovery uses numeric equality, for which IEEE `-0.0` and
/// `+0.0` denote the same zero-valued scientific result.
///
/// # Errors
///
/// Returns input errors for non-finite values and configuration errors for
/// `k < 0` or negative `standard_error`.
pub fn accept_within_standard_errors(
    estimate: f64,
    target: f64,
    standard_error: f64,
    k: f64,
) -> Result<bool, ValidationError> {
    if ![estimate, target, standard_error, k]
        .iter()
        .all(|value| value.is_finite())
    {
        return Err(ValidationError::InvalidInput);
    }
    if k < 0.0 || standard_error < 0.0 {
        return Err(ValidationError::InvalidConfiguration);
    }
    if standard_error == 0.0 || k == 0.0 {
        // Exact recovery is numerical equality; signed zero is one zero value.
        return Ok(estimate == target);
    }

    let direct_error = estimate - target;
    let direct_bound = k * standard_error;
    if direct_error.is_finite() {
        if direct_bound.is_finite() {
            let residual = direct_error.abs();
            if residual == direct_bound && residual != 0.0 {
                let difference_roundoff = subtraction_roundoff(estimate, target, direct_error);
                let residual_roundoff = if direct_error.is_sign_negative() {
                    -difference_roundoff
                } else {
                    difference_roundoff
                };
                let product_roundoff = k.mul_add(standard_error, -direct_bound);
                if residual_roundoff != product_roundoff {
                    return Ok(residual_roundoff < product_roundoff);
                }
                if residual_roundoff == 0.0 && direct_bound.is_subnormal() {
                    return Ok(represented_magnitude_le_exact_product(
                        residual,
                        k,
                        standard_error,
                    ));
                }
            }
            return Ok(residual <= direct_bound);
        }
        // A finite residual is necessarily inside a positive bound whose
        // represented multiplication overflowed beyond binary64's finite range.
        return Ok(true);
    }
    if direct_bound.is_finite() {
        // The represented residual overflowed while the positive bound did not.
        return Ok(false);
    }

    Ok(both_overflow_acceptance(
        estimate,
        target,
        standard_error,
        k,
    ))
}

#[allow(clippy::cast_possible_truncation)]
fn nearest_rank(sorted: &[f64], percentile: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = (percentile * sorted.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[index]
}

#[cfg(test)]
mod tests {
    use super::{MonteCarloSummary, accept_within_standard_errors, summarize_replications};
    use crate::ValidationError;

    #[test]
    fn monte_carlo_summary_and_acceptance_gates() {
        let samples = [1.0, 2.0, 3.0, 4.0];
        let summary = summarize_replications(&samples, 0.25, 0.75).expect("sum");
        assert_eq!(summary.replication_count, 4);
        assert!((summary.mean - 2.5).abs() < 1e-12);
        assert!(summary.standard_deviation > 0.0);
        assert!(summary.standard_error > 0.0);
        assert!((summary.percentile_lower - 1.0).abs() < 1e-12);
        assert!((summary.percentile_upper - 3.0).abs() < 1e-12);
        let single = summarize_replications(&[2.0], 0.0, 1.0).expect("one");
        assert!((single.standard_deviation - 0.0).abs() < 1e-12);
        assert!((single.percentile_lower - 2.0).abs() < 1e-12);
        assert!(accept_within_standard_errors(1.0, 1.0, 0.1, 1.0).expect("acc"));
        assert!(!accept_within_standard_errors(1.0, 2.0, 0.1, 1.0).expect("rej"));
        assert_eq!(
            summarize_replications(&[], 0.1, 0.9),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            summarize_replications(&[f64::NAN], 0.1, 0.9),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            summarize_replications(&[1.0], 0.9, 0.1),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            summarize_replications(&[1.0], -0.1, 0.5),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            summarize_replications(&[1.0], 0.0, 1.1),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            accept_within_standard_errors(1.0, 1.0, -0.1, 1.0),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            accept_within_standard_errors(1.0, 1.0, 0.1, -1.0),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            accept_within_standard_errors(f64::NAN, 1.0, 0.1, 1.0),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            accept_within_standard_errors(1.0, f64::INFINITY, 0.1, 1.0),
            Err(ValidationError::InvalidInput)
        );
        // equal percentiles
        let edge = summarize_replications(&[1.0, 2.0], 0.5, 0.5).expect("eq");
        assert!((edge.percentile_lower - edge.percentile_upper).abs() < 1e-12);
        // Large finite samples must not overflow the summary path.
        let large = summarize_replications(&[f64::MAX, f64::MAX], 0.0, 1.0).expect("large");
        assert!((large.mean - f64::MAX).abs() < 1.0);
        assert!((large.standard_deviation - 0.0).abs() < 1e-12);
        assert!(
            !accept_within_standard_errors(f64::MAX, -f64::MAX, f64::MAX, 1.5).expect("scaled")
        );
    }

    #[test]
    fn nonfinite_acceptance_and_summary_validate() {
        assert!(accept_within_standard_errors(1.0, 1.0, 0.0, 1.0).expect("eq"));
        assert!(!accept_within_standard_errors(1.0, 2.0, 0.0, 1.0).expect("neq"));
        assert_eq!(
            summarize_replications(&[f64::MAX, -f64::MAX], 0.0, 1.0),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 0,
                mean: 0.0,
                standard_deviation: 0.0,
                standard_error: 0.0,
                percentile_lower: 0.0,
                percentile_upper: 1.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 2,
                mean: f64::NAN,
                standard_deviation: 0.0,
                standard_error: 0.0,
                percentile_lower: 0.0,
                percentile_upper: 1.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 2,
                mean: 0.0,
                standard_deviation: -0.1,
                standard_error: 0.0,
                percentile_lower: 0.0,
                percentile_upper: 1.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 2,
                mean: 0.0,
                standard_deviation: 0.0,
                standard_error: -0.1,
                percentile_lower: 0.0,
                percentile_upper: 1.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 2,
                mean: 0.0,
                standard_deviation: 0.0,
                standard_error: 0.0,
                percentile_lower: 1.0,
                percentile_upper: 0.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
    }
}
