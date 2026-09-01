//! Event-time lagged association standardization.
//!
//! This module belongs to the Longitudinal Modeling bounded context. It
//! standardizes a lagged covariance only when both marginal variances are
//! available. A one-sided covariance/initial-variance ratio is deliberately
//! not exposed as an autocorrelation.

use crate::{EventTimeInterval, LongitudinalError};

/// Decompose a positive finite binary64 value into an exact integer
/// significand and a power-of-two exponent.
fn positive_binary_components(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    if exponent_bits == 0 {
        (fraction, -1074)
    } else {
        ((1_u64 << 52) | fraction, exponent_bits - 1023 - 52)
    }
}

/// Compare two positive `u128 * 2^exponent` values without overflowing the
/// integer significands.
fn scaled_integer_leq(
    left_significand: u128,
    left_exponent: i32,
    right_significand: u128,
    right_exponent: i32,
) -> bool {
    let left_bits = (u128::BITS - left_significand.leading_zeros()) as i32;
    let right_bits = (u128::BITS - right_significand.leading_zeros()) as i32;
    let left_order = left_exponent + left_bits;
    let right_order = right_exponent + right_bits;
    if left_order != right_order {
        return left_order < right_order;
    }
    if left_exponent == right_exponent {
        return left_significand <= right_significand;
    }
    if left_exponent > right_exponent {
        let shift = (left_exponent - right_exponent) as u32;
        return (left_significand << shift) <= right_significand;
    }
    let shift = (right_exponent - left_exponent) as u32;
    left_significand <= (right_significand << shift)
}

/// Test the Cauchy–Schwarz covariance bound exactly for the supplied binary64
/// inputs rather than using a rounded floating-point square-root product.
fn covariance_within_binary_bound(
    lagged_covariance: f64,
    earlier_total_variance: f64,
    later_total_variance: f64,
) -> bool {
    let covariance_magnitude = lagged_covariance.abs();
    if covariance_magnitude == 0.0 {
        return true;
    }
    let (covariance_significand, covariance_exponent) =
        positive_binary_components(covariance_magnitude);
    let (earlier_significand, earlier_exponent) =
        positive_binary_components(earlier_total_variance);
    let (later_significand, later_exponent) =
        positive_binary_components(later_total_variance);

    let covariance_square =
        u128::from(covariance_significand) * u128::from(covariance_significand);
    let variance_product =
        u128::from(earlier_significand) * u128::from(later_significand);
    scaled_integer_leq(
        covariance_square,
        covariance_exponent * 2,
        variance_product,
        earlier_exponent + later_exponent,
    )
}

/// Recover a Pearson correlation for an event-time lag from its covariance and
/// both marginal variances.
///
/// For observations at event times `t` and `t + Δ`, the correlation is
///
/// `Cov(Y_t, Y_{t+Δ}) / sqrt(Var(Y_t) * Var(Y_{t+Δ}))`.
///
/// Requiring both marginals is essential for nonstationary processes. Driver,
/// Oud, and Voelkle (2017) provide the continuous-time state-transition and
/// covariance components from which occasion-specific marginals can be built;
/// they do not justify replacing the second marginal variance with the first
/// when the process is nonstationary.
///
/// [`EventTimeInterval`] makes substantive event-time ownership explicit. This
/// function does not infer either marginal variance and does not estimate a
/// state process.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidTemporalAssociationInput`] for
/// non-finite covariance or marginal inputs,
/// [`LongitudinalError::NonPositiveMarginalVariance`] when either marginal
/// variance is not strictly positive, and
/// [`LongitudinalError::CovarianceBoundViolation`] when the supplied covariance
/// is incompatible with the two marginal variances.
pub(crate) fn recover_event_time_lagged_correlation(
    lagged_covariance: f64,
    earlier_total_variance: f64,
    later_total_variance: f64,
    _event_interval: EventTimeInterval,
) -> Result<f64, LongitudinalError> {
    if !lagged_covariance.is_finite()
        || !earlier_total_variance.is_finite()
        || !later_total_variance.is_finite()
    {
        return Err(LongitudinalError::InvalidTemporalAssociationInput);
    }
    if earlier_total_variance <= 0.0 || later_total_variance <= 0.0 {
        return Err(LongitudinalError::NonPositiveMarginalVariance);
    }
    if !covariance_within_binary_bound(
        lagged_covariance,
        earlier_total_variance,
        later_total_variance,
    ) {
        return Err(LongitudinalError::CovarianceBoundViolation);
    }

    let earlier_scale = earlier_total_variance.sqrt();
    let later_scale = later_total_variance.sqrt();
    // Divide by the smaller scale first. The exact covariance-bound gate above
    // guarantees the intermediate magnitude cannot exceed the remaining
    // marginal scale, while this order avoids underflow when the marginals are
    // separated by hundreds of binary exponents.
    let (first_scale, second_scale) = if earlier_scale <= later_scale {
        (earlier_scale, later_scale)
    } else {
        (later_scale, earlier_scale)
    };
    let correlation = (lagged_covariance / first_scale) / second_scale;

    // Finite positive marginals plus the exact covariance-bound gate guarantee
    // that both divisions stay finite. Clamping only absorbs final
    // square-root/division rounding at a valid ±1 boundary; it cannot admit an
    // over-bound covariance.
    Ok(correlation.clamp(-1.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::{recover_event_time_lagged_correlation, scaled_integer_leq};
    use crate::{EventTimeInterval, LongitudinalError};

    fn event_time(value: f64) -> EventTimeInterval {
        EventTimeInterval::new(value).expect("test interval must be valid event time")
    }

    #[test]
    fn nonstationary_marginals_do_not_use_the_earlier_variance_twice() {
        let recovered = recover_event_time_lagged_correlation(1.5, 1.0, 4.0, event_time(1.0))
            .expect("valid nonstationary covariance should standardize");
        assert!((recovered - 0.75).abs() < f64::EPSILON * 4.0);
    }

    #[test]
    fn covariance_bound_is_fail_closed() {
        assert_eq!(
            recover_event_time_lagged_correlation(2.000_000_000_1, 1.0, 4.0, event_time(1.0)),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(-2.000_000_000_1, 1.0, 4.0, event_time(1.0)),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
    }

    #[test]
    fn exact_binary_bound_rejects_one_ulp_excess_for_both_signs() {
        let variance = 2.0_f64;
        let one_ulp_above = f64::from_bits(variance.to_bits() + 1);
        assert_eq!(
            recover_event_time_lagged_correlation(
                one_ulp_above,
                variance,
                variance,
                event_time(1.0),
            ),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(
                -one_ulp_above,
                variance,
                variance,
                event_time(1.0),
            ),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
    }

    #[test]
    fn exact_binary_bound_accepts_extreme_and_subnormal_boundaries() {
        assert_eq!(
            recover_event_time_lagged_correlation(
                f64::MAX,
                f64::MAX,
                f64::MAX,
                event_time(1.0),
            ),
            Ok(1.0)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(
                -f64::MAX,
                f64::MAX,
                f64::MAX,
                event_time(1.0),
            ),
            Ok(-1.0)
        );
        let minimum_subnormal = f64::from_bits(1);
        assert_eq!(
            recover_event_time_lagged_correlation(
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                event_time(1.0),
            ),
            Ok(1.0)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(
                -minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                event_time(1.0),
            ),
            Ok(-1.0)
        );
    }

    #[test]
    fn gross_subnormal_bound_violation_is_classified_before_division() {
        let minimum_subnormal = f64::from_bits(1);
        assert_eq!(
            recover_event_time_lagged_correlation(
                1.0,
                minimum_subnormal,
                minimum_subnormal,
                event_time(1.0),
            ),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
    }

    #[test]
    fn unequal_scales_do_not_underflow_a_representable_correlation() {
        let recovered = recover_event_time_lagged_correlation(
            f64::MIN_POSITIVE,
            f64::MIN_POSITIVE,
            f64::MAX,
            event_time(1.0),
        )
        .expect("valid unequal-scale covariance should remain representable");
        assert!(recovered > 0.0);
        assert!(recovered.is_finite());
    }

    #[test]
    fn scale_order_is_symmetric_when_the_earlier_marginal_is_larger() {
        let recovered = recover_event_time_lagged_correlation(1.5, 4.0, 1.0, event_time(1.0))
            .expect("reversing marginal scale order should still standardize");
        assert!((recovered - 0.75).abs() < f64::EPSILON * 4.0);
    }

    #[test]
    fn zero_covariance_is_valid() {
        assert_eq!(
            recover_event_time_lagged_correlation(0.0, 1.0, 4.0, event_time(1.0)),
            Ok(0.0)
        );
    }

    #[test]
    fn scaled_integer_comparison_covers_alignment_directions() {
        assert!(scaled_integer_leq(1, 2, 4, 0));
        assert!(!scaled_integer_leq(5, 0, 1, 2));
        assert!(scaled_integer_leq(3, 0, 6, -1));
        assert!(scaled_integer_leq(6, -1, 3, 0));
        assert!(!scaled_integer_leq(7, -1, 3, 0));
    }

    #[test]
    fn exact_boundary_correlations_are_allowed() {
        assert_eq!(
            recover_event_time_lagged_correlation(2.0, 1.0, 4.0, event_time(1.0)),
            Ok(1.0)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(-2.0, 1.0, 4.0, event_time(1.0)),
            Ok(-1.0)
        );
    }

    #[test]
    fn square_root_scaling_avoids_avoidable_variance_product_overflow() {
        let variance = f64::MAX / 4.0;
        let recovered = recover_event_time_lagged_correlation(
            variance / 2.0,
            variance,
            variance,
            event_time(1.0),
        )
        .expect("representable standardized covariance should remain representable");
        assert!((recovered - 0.5).abs() < 1.0e-15);
    }

    #[test]
    fn every_non_finite_input_position_fails_closed() {
        assert_eq!(
            recover_event_time_lagged_correlation(f64::NAN, 1.0, 1.0, event_time(1.0)),
            Err(LongitudinalError::InvalidTemporalAssociationInput)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(0.0, f64::INFINITY, 1.0, event_time(1.0)),
            Err(LongitudinalError::InvalidTemporalAssociationInput)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(0.0, 1.0, f64::INFINITY, event_time(1.0)),
            Err(LongitudinalError::InvalidTemporalAssociationInput)
        );
    }

    #[test]
    fn either_non_positive_marginal_fails_closed() {
        assert_eq!(
            recover_event_time_lagged_correlation(0.0, 0.0, 1.0, event_time(1.0)),
            Err(LongitudinalError::NonPositiveMarginalVariance)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(0.0, 1.0, 0.0, event_time(1.0)),
            Err(LongitudinalError::NonPositiveMarginalVariance)
        );
    }
}
