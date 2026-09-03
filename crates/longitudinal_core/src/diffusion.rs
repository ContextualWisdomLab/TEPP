//! Scalar diffusion standardisation on substantive event time.
//!
//! These functions preserve the scientific evidence from TEPP PRs #476 and
//! #477 while moving temporal/state composition out of `psychometric_core`.
//! Driver, Oud, and Voelkle (2017) print the underlying continuous/discrete
//! diffusion transformations and describe relevant-variance standardisation,
//! but the 2017 ctsem summary source does not emit named `DIFFUSIONstd` or
//! `discreteDIFFUSIONstd` matrices. The scalar maps below are therefore
//! research-candidate extensions, not canonical ctsem output and not a DSEM or
//! ctsem estimator.

use crate::{EventTimeInterval, LongitudinalError, stationary::validate_stationary_process_inputs};

/// Recover the scalar research-candidate `DIFFUSIONstd = q / p` map.
///
/// `q` is the continuous diffusion variance-rate input and `p` is the strictly
/// positive stationary within-person variance `-q/(2a)`. For every finite
/// positive `q` and stable finite `a`, that real-valued variance exists and the
/// scalar standardisation cancels it exactly to `-2a`. The implementation
/// therefore validates the stationary process without materializing `p` as
/// binary64; an intermediate `p` below or above the `f64` range must not erase
/// a representable final standardized ratio. Equal numeric values still do not
/// collapse this estimand into `asymDIFFUSIONstd` or another variance
/// standardisation.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidTemporalTransformInput`] for non-finite
/// inputs, negative diffusion, or a non-representable final `-2a` ratio. Returns
/// [`LongitudinalError::StationaryVarianceRequiresStableDrift`] unless `a < 0`.
/// Returns [`LongitudinalError::StandardisedDiffusionRequiresPositiveWithinVariance`]
/// when continuous diffusion is exactly zero, because the stationary variance
/// is then zero rather than merely outside the binary64 range.
pub fn recover_event_time_standardised_continuous_diffusion(
    continuous_diffusion: f64,
    log_rate: f64,
) -> Result<f64, LongitudinalError> {
    validate_stationary_process_inputs(continuous_diffusion, log_rate)?;
    if continuous_diffusion == 0.0 {
        return Err(LongitudinalError::StandardisedDiffusionRequiresPositiveWithinVariance);
    }

    // Algebraically q / (-q/(2a)) == -2a for every positive q. The stationary
    // process admission above establishes that cancellation in the real-valued
    // model without requiring the cancelled p itself to fit in binary64.
    let ratio = -2.0 * log_rate;
    if !ratio.is_finite() {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }
    Ok(ratio)
}

/// Recover the scalar research-candidate `discreteDIFFUSIONstd = Q_delta / p` map.
///
/// For a stable scalar continuous-time process, dividing discrete process noise
/// over an event interval by stationary within-person variance yields
/// `1 - exp(2 a delta)`. The implementation evaluates this ratio directly with
/// `exp_m1` after proving positive stationary variance algebraically from finite
/// positive diffusion and stable drift. It does not materialize the cancelled
/// stationary variance, so a below-range or above-range `p` cannot reject a
/// representable final ratio. The [`EventTimeInterval`] value object prevents
/// measurement occasion, document, assertion, system, or availability durations
/// from being passed as event time accidentally.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidTemporalTransformInput`] for invalid
/// diffusion/drift, a doubled event-time exponent that underflows to signed
/// zero or overflows, or a finite-interval final ratio that collapses to the
/// false binary64 endpoints zero or one. Returns
/// [`LongitudinalError::StationaryVarianceRequiresStableDrift`] unless `a < 0`.
/// Returns [`LongitudinalError::StandardisedDiffusionRequiresPositiveWithinVariance`]
/// when continuous diffusion is exactly zero.
pub fn recover_event_time_standardised_discrete_diffusion(
    continuous_diffusion: f64,
    log_rate: f64,
    event_interval: EventTimeInterval,
) -> Result<f64, LongitudinalError> {
    validate_stationary_process_inputs(continuous_diffusion, log_rate)?;
    if continuous_diffusion == 0.0 {
        return Err(LongitudinalError::StandardisedDiffusionRequiresPositiveWithinVariance);
    }

    let interval = event_interval.as_f64();
    let doubled_interval = interval * 2.0;
    let exponent = if doubled_interval.is_finite() {
        // Multiplication by two is exact while finite. Scaling the interval
        // before the only rounded product preserves a representable `2aΔ`
        // when `aΔ` alone would round to signed zero, and it avoids forming
        // `2a`, whose intermediate can overflow for an extreme stable drift.
        log_rate * doubled_interval
    } else {
        // If 2Δ overflows, Δ is already enormous. Form aΔ first and then apply
        // the exact factor two; this branch cannot suffer the tiny-interval
        // underflow that motivated the primary ordering above. A non-finite
        // product cannot be reported as the exact finite-interval endpoint one.
        let half_exponent = log_rate * interval;
        if !half_exponent.is_finite() {
            return Err(LongitudinalError::InvalidTemporalTransformInput);
        }
        half_exponent * 2.0
    };

    // For finite stable a and finite positive Δ, the exact 2aΔ is finite and
    // strictly negative. Signed zero or infinity therefore means the target
    // exponent itself is not representable in binary64 and must fail closed.
    if !exponent.is_finite() || exponent == 0.0 {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }

    let ratio = -exponent.exp_m1();
    // The exact finite-interval ratio is strictly inside (0, 1). If exp_m1
    // rounds it to either endpoint, reporting zero/one would erase a nonzero
    // remainder and turn a numerical limitation into a scientific boundary.
    if !ratio.is_finite() || ratio <= 0.0 || ratio >= 1.0 {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }
    Ok(ratio)
}

/// Refuse treating a continuous standardised diffusion ratio as a discrete one.
///
/// `q/p` is interval-independent while `Q_delta/p` is an event-interval
/// quantity. Numerical equality at a particular parameter value does not make
/// the named estimands interchangeable.
///
/// # Errors
///
/// Always returns [`LongitudinalError::ContinuousDiffusionIsNotDiscreteDiffusion`].
pub fn refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion(
    continuous_standardised_diffusion: f64,
    discrete_standardised_diffusion: f64,
) -> Result<f64, LongitudinalError> {
    let _ = (
        continuous_standardised_diffusion,
        discrete_standardised_diffusion,
    );
    Err(LongitudinalError::ContinuousDiffusionIsNotDiscreteDiffusion)
}

/// Refuse treating an unstandardised diffusion quantity as a standardised one.
///
/// # Errors
///
/// Always returns [`LongitudinalError::UnstandardisedDiffusionIsNotStandardisedDiffusion`].
pub fn refuse_unstandardised_diffusion_as_standardised_diffusion(
    unstandardised_diffusion: f64,
    standardised_diffusion: f64,
) -> Result<f64, LongitudinalError> {
    let _ = (unstandardised_diffusion, standardised_diffusion);
    Err(LongitudinalError::UnstandardisedDiffusionIsNotStandardisedDiffusion)
}

/// Refuse scaling diffusion by total trait-plus-state variance as standardisation.
///
/// Driver et al.'s relevant-variance rule for these scalar research candidates
/// uses within-person stationary variance, not a total that also contains
/// between-unit trait or added time-independent-predictor variance.
///
/// # Errors
///
/// Always returns [`LongitudinalError::TotalVarianceScaledDiffusionIsNotStandardisedDiffusion`].
pub fn refuse_total_variance_scaled_diffusion_as_standardised_diffusion(
    total_variance_scaled_diffusion: f64,
    standardised_diffusion: f64,
) -> Result<f64, LongitudinalError> {
    let _ = (total_variance_scaled_diffusion, standardised_diffusion);
    Err(LongitudinalError::TotalVarianceScaledDiffusionIsNotStandardisedDiffusion)
}

#[cfg(test)]
mod tests {
    use super::{
        recover_event_time_standardised_continuous_diffusion,
        recover_event_time_standardised_discrete_diffusion,
    };
    use crate::{EventTimeInterval, LongitudinalError};

    #[test]
    fn discrete_candidate_preserves_representable_minimum_subnormal_ratio() {
        let tiny = EventTimeInterval::new(f64::from_bits(1)).expect("minimum subnormal interval");
        let recovered = recover_event_time_standardised_discrete_diffusion(1.0, -0.5, tiny)
            .expect("the final standardized ratio is the minimum positive subnormal");
        assert_eq!(recovered.to_bits(), 1);
    }

    #[test]
    fn finite_event_interval_must_not_collapse_to_false_unit_diffusion() {
        let huge = EventTimeInterval::new(f64::MAX).expect("finite positive interval");
        assert_eq!(
            recover_event_time_standardised_discrete_diffusion(1.0, -1.0, huge),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );

        let finite_saturating = EventTimeInterval::new(50.0).expect("finite event interval");
        assert_eq!(
            recover_event_time_standardised_discrete_diffusion(1.0, -1.0, finite_saturating),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }

    #[test]
    fn standardised_diffusion_does_not_materialise_a_cancelled_stationary_variance() {
        let minimum_subnormal = f64::from_bits(1);
        let continuous_underflow =
            recover_event_time_standardised_continuous_diffusion(minimum_subnormal, -1.0)
                .expect("q/p cancels a positive real stationary variance below binary64 range");
        assert_eq!(continuous_underflow, 2.0);

        let continuous_overflow =
            recover_event_time_standardised_continuous_diffusion(f64::MAX, -0.25)
                .expect("q/p cancels a positive real stationary variance above binary64 range");
        assert_eq!(continuous_overflow, 0.5);

        let interval = EventTimeInterval::new(1.0).expect("unit event interval");
        let discrete_underflow = recover_event_time_standardised_discrete_diffusion(
            minimum_subnormal,
            -1.0,
            interval,
        )
        .expect("Q_delta/p cancels the unrepresentable stationary intermediate");
        let underflow_truth = -(-2.0_f64).exp_m1();
        assert!((discrete_underflow - underflow_truth).abs() <= f64::EPSILON);

        let discrete_overflow =
            recover_event_time_standardised_discrete_diffusion(f64::MAX, -0.25, interval)
                .expect("finite standardized discrete diffusion must survive p overflow");
        let overflow_truth = -(-0.5_f64).exp_m1();
        assert!((discrete_overflow - overflow_truth).abs() <= f64::EPSILON);
    }

    #[test]
    fn continuous_candidate_rejects_nonrepresentable_ratio() {
        assert_eq!(
            recover_event_time_standardised_continuous_diffusion(f64::MAX, -f64::MAX),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }
}
