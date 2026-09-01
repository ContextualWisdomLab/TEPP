//! Event-time lagged association standardization.
//!
//! This module belongs to the Longitudinal Modeling bounded context. It
//! standardizes a lagged covariance only when both marginal variances are
//! available. A one-sided covariance/initial-variance ratio is deliberately
//! not exposed as an autocorrelation.

use crate::LongitudinalError;

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
/// `event_interval` is semantically required to be strictly positive so a
/// measurement-occasion or method facet cannot be passed as an untyped lag.
/// This function does not infer either marginal variance and does not estimate
/// a state process.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidTemporalAssociationInput`] for
/// non-finite inputs, [`LongitudinalError::NonPositiveMarginalVariance`] when
/// either marginal variance is not strictly positive,
/// [`LongitudinalError::NonPositiveEventInterval`] for a non-positive event
/// interval, and [`LongitudinalError::CovarianceBoundViolation`] when the
/// supplied covariance is incompatible with the two marginal variances.
pub fn recover_event_time_lagged_correlation(
    lagged_covariance: f64,
    earlier_total_variance: f64,
    later_total_variance: f64,
    event_interval: f64,
) -> Result<f64, LongitudinalError> {
    if !lagged_covariance.is_finite()
        || !earlier_total_variance.is_finite()
        || !later_total_variance.is_finite()
        || !event_interval.is_finite()
    {
        return Err(LongitudinalError::InvalidTemporalAssociationInput);
    }
    if earlier_total_variance <= 0.0 || later_total_variance <= 0.0 {
        return Err(LongitudinalError::NonPositiveMarginalVariance);
    }
    if event_interval <= 0.0 {
        return Err(LongitudinalError::NonPositiveEventInterval);
    }

    // sqrt(v1) * sqrt(v2) avoids the avoidable overflow of sqrt(v1 * v2).
    let denominator = earlier_total_variance.sqrt() * later_total_variance.sqrt();
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(LongitudinalError::InvalidTemporalAssociationInput);
    }
    let correlation = lagged_covariance / denominator;
    if !correlation.is_finite() {
        return Err(LongitudinalError::InvalidTemporalAssociationInput);
    }
    if correlation.abs() > 1.0 {
        return Err(LongitudinalError::CovarianceBoundViolation);
    }
    Ok(correlation)
}

#[cfg(test)]
mod tests {
    use super::recover_event_time_lagged_correlation;
    use crate::LongitudinalError;

    #[test]
    fn nonstationary_marginals_do_not_use_the_earlier_variance_twice() {
        // The retired one-sided ratio would be 1.5 and therefore impossible as
        // an autocorrelation. Supplying the later marginal gives 0.75.
        let recovered = recover_event_time_lagged_correlation(1.5, 1.0, 4.0, 1.0)
            .expect("valid nonstationary covariance should standardize");
        assert!((recovered - 0.75).abs() < f64::EPSILON * 4.0);
    }

    #[test]
    fn covariance_bound_is_fail_closed() {
        assert_eq!(
            recover_event_time_lagged_correlation(2.000_000_000_1, 1.0, 4.0, 1.0),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(-2.000_000_000_1, 1.0, 4.0, 1.0),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
    }

    #[test]
    fn exact_binary_bound_rejects_one_ulp_excess_for_both_signs() {
        let variance = 2.0_f64;
        let one_ulp_above = f64::from_bits(variance.to_bits() + 1);
        assert_eq!(
            recover_event_time_lagged_correlation(one_ulp_above, variance, variance, 1.0),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(-one_ulp_above, variance, variance, 1.0),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
    }

    #[test]
    fn exact_binary_bound_accepts_extreme_and_subnormal_boundaries() {
        assert_eq!(
            recover_event_time_lagged_correlation(f64::MAX, f64::MAX, f64::MAX, 1.0),
            Ok(1.0)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(-f64::MAX, f64::MAX, f64::MAX, 1.0),
            Ok(-1.0)
        );

        let minimum_subnormal = f64::from_bits(1);
        assert_eq!(
            recover_event_time_lagged_correlation(
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                1.0,
            ),
            Ok(1.0)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(
                -minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                1.0,
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
                1.0,
            ),
            Err(LongitudinalError::CovarianceBoundViolation)
        );
    }

    #[test]
    fn exact_boundary_correlations_are_allowed() {
        assert_eq!(
            recover_event_time_lagged_correlation(2.0, 1.0, 4.0, 1.0),
            Ok(1.0)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(-2.0, 1.0, 4.0, 1.0),
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
            1.0,
        )
        .expect("representable standardized covariance should remain representable");
        assert!((recovered - 0.5).abs() < 1.0e-15);
    }

    #[test]
    fn invalid_inputs_fail_closed() {
        assert_eq!(
            recover_event_time_lagged_correlation(f64::NAN, 1.0, 1.0, 1.0),
            Err(LongitudinalError::InvalidTemporalAssociationInput)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(0.0, 0.0, 1.0, 1.0),
            Err(LongitudinalError::NonPositiveMarginalVariance)
        );
        assert_eq!(
            recover_event_time_lagged_correlation(0.0, 1.0, 1.0, 0.0),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
    }
}