//! Event-time standardisation for scalar continuous-time drift.

use crate::{
    EventTimeInterval, LongitudinalError, stationary::recover_stationary_within_variance,
};

/// Recover the scalar p. 16 `discreteDRIFTstd` on event time.
///
/// Driver, Oud, and Voelkle (2017) define the discrete-time transition over an
/// interval as `exp(A * delta_t)`. Their standardisation uses the relevant
/// within-person asymptotic variance. In the scalar stationary case the
/// affecting/affected standard-deviation ratio is one, so the standardised
/// auto-effect is numerically `exp(a * delta_t)` after a strictly positive
/// stationary within-person variance has been established. Equal numerical
/// values do not make unstandardised `discreteDRIFT` and `discreteDRIFTstd`
/// the same estimand.
///
/// This function is temporal composition, not a ctsem/DSEM estimator. It does
/// not estimate `a`, process noise, uncertainty, or a latent state. The caller
/// supplies the continuous diffusion intensity, stable scalar drift, and an
/// [`EventTimeInterval`] admitted by the Longitudinal Modeling bounded context.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidTemporalTransformInput`] for non-finite
/// diffusion/drift inputs, negative diffusion, non-representable stationary
/// variance, an `a * delta_t` product that overflows or underflows to signed
/// zero, or an exponential that underflows to zero. Returns
/// [`LongitudinalError::StationaryVarianceRequiresStableDrift`] unless `a < 0`.
/// Returns [`LongitudinalError::StandardisedDriftRequiresPositiveWithinVariance`]
/// when the stationary within-person variance is zero or underflows to zero.
pub fn recover_event_time_standardised_discrete_drift(
    continuous_diffusion: f64,
    log_rate: f64,
    event_interval: EventTimeInterval,
) -> Result<f64, LongitudinalError> {
    let within_person_variance =
        recover_stationary_within_variance(continuous_diffusion, log_rate)?;
    if within_person_variance <= 0.0 {
        return Err(LongitudinalError::StandardisedDriftRequiresPositiveWithinVariance);
    }

    let exponent = log_rate * event_interval.as_f64();
    // `log_rate` is strictly negative and EventTimeInterval is strictly
    // positive, so an exact product cannot be zero. A signed zero therefore
    // proves binary64 multiplication underflow and must fail closed instead of
    // silently becoming exp(-0.0) == 1.0.
    if !exponent.is_finite() || exponent == 0.0 {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }
    let discrete_drift = exponent.exp();
    if discrete_drift == 0.0 {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }
    Ok(discrete_drift)
}

/// Refuse treating unstandardised `discreteDRIFT` as `discreteDRIFTstd`.
///
/// The scalar stationary values may coincide numerically while the named
/// quantities and their admissibility conditions remain distinct.
///
/// # Errors
///
/// Always returns [`LongitudinalError::UnstandardisedDriftIsNotStandardisedDrift`].
pub fn refuse_unstandardised_discrete_drift_as_standardised_discrete_drift(
    unstandardised_discrete_drift: f64,
    standardised_discrete_drift: f64,
) -> Result<f64, LongitudinalError> {
    let _ = (unstandardised_discrete_drift, standardised_discrete_drift);
    Err(LongitudinalError::UnstandardisedDriftIsNotStandardisedDrift)
}

/// Refuse treating a trait-plus-state lagged association as `discreteDRIFTstd`.
///
/// A trait-plus-state association mixes stable between-unit variance with
/// within-person dynamics. Driver et al.'s drift standardisation uses the
/// relevant within-person variance instead.
///
/// # Errors
///
/// Always returns [`LongitudinalError::TraitStateAssociationIsNotStandardisedDrift`].
pub fn refuse_trait_plus_state_association_as_standardised_discrete_drift(
    trait_plus_state_association: f64,
    standardised_discrete_drift: f64,
) -> Result<f64, LongitudinalError> {
    let _ = (trait_plus_state_association, standardised_discrete_drift);
    Err(LongitudinalError::TraitStateAssociationIsNotStandardisedDrift)
}

/// Refuse using between-unit trait variance as the drift standardisation variance.
///
/// # Errors
///
/// Always returns [`LongitudinalError::TraitVarianceIsNotDriftStandardisationVariance`].
pub fn refuse_trait_variance_as_standardisation_variance(
    trait_variance: f64,
    within_person_variance: f64,
) -> Result<f64, LongitudinalError> {
    let _ = (trait_variance, within_person_variance);
    Err(LongitudinalError::TraitVarianceIsNotDriftStandardisationVariance)
}
