//! Internal stationary within-person variance arithmetic.

use crate::LongitudinalError;

/// Recover scalar stationary within-person variance `p = -q / (2a)`.
///
/// This primitive is intentionally private to Longitudinal Modeling. It admits
/// finite non-negative continuous diffusion and strictly negative drift while
/// avoiding an otherwise unnecessary overflow in the intermediate `2a`.
/// Callers decide whether zero stationary variance is admissible for their
/// named estimand.
pub(crate) fn recover_stationary_within_variance(
    continuous_diffusion: f64,
    log_rate: f64,
) -> Result<f64, LongitudinalError> {
    if !continuous_diffusion.is_finite() || continuous_diffusion < 0.0 || !log_rate.is_finite() {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }
    if log_rate >= 0.0 {
        return Err(LongitudinalError::StationaryVarianceRequiresStableDrift);
    }
    if continuous_diffusion == 0.0 {
        return Ok(0.0);
    }

    let twice_rate = log_rate * 2.0;
    let stationary = if twice_rate.is_finite() {
        continuous_diffusion / -twice_rate
    } else {
        // Scale the overflowing denominator down before division and compensate
        // afterward. This keeps a representable minimum-subnormal result from
        // being rounded to one subnormal and then erased by a final halving.
        (continuous_diffusion / (-log_rate * 0.5)) * 0.25
    };
    if !stationary.is_finite() {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }
    Ok(stationary)
}

#[cfg(test)]
mod tests {
    use super::recover_stationary_within_variance;
    use crate::LongitudinalError;

    #[test]
    fn avoids_doubling_overflow_when_final_stationary_variance_is_representable() {
        let stationary = recover_stationary_within_variance(f64::MAX, -f64::MAX)
            .expect("q / (-2a) remains representable");
        assert_eq!(stationary, 0.5);
    }

    #[test]
    fn preserves_minimum_subnormal_stationary_variance_after_drift_overflow() {
        // Exact q / (-2a) lies above the half-ulp threshold for the minimum
        // positive subnormal. Dividing by |a| first rounds to one subnormal;
        // halving that rounded intermediate incorrectly erases the result.
        let diffusion = 1.332_267_629_550_187_7e-15_f64;
        let stationary = recover_stationary_within_variance(diffusion, -f64::MAX)
            .expect("positive subnormal stationary variance is representable");
        assert_eq!(stationary.to_bits(), 1);
    }

    #[test]
    fn stationary_variance_admission_is_fail_closed() {
        assert_eq!(recover_stationary_within_variance(0.0, -0.5), Ok(0.0));
        assert_eq!(
            recover_stationary_within_variance(1.0, 0.0),
            Err(LongitudinalError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_within_variance(-1.0, -0.5),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }
}
