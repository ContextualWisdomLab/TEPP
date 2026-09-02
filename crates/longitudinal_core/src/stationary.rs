//! Internal stationary within-person variance arithmetic.

use crate::LongitudinalError;

/// Validate inputs for a scalar stationary continuous-time process.
///
/// Finite `q >= 0` and strictly negative finite drift establish a finite,
/// non-negative stationary variance in the mathematical real-valued model.
/// This admission deliberately does not materialize `q / (-2a)` as binary64:
/// standardized scalar maps may cancel that variance algebraically even when
/// the intermediate itself lies below or above the representable `f64` range.
pub(crate) fn validate_stationary_process_inputs(
    continuous_diffusion: f64,
    log_rate: f64,
) -> Result<(), LongitudinalError> {
    if !continuous_diffusion.is_finite() || continuous_diffusion < 0.0 || !log_rate.is_finite() {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }
    if log_rate >= 0.0 {
        return Err(LongitudinalError::StationaryVarianceRequiresStableDrift);
    }
    Ok(())
}

/// Recover scalar stationary within-person variance `p = -q / (2a)`.
///
/// This primitive is intentionally private to Longitudinal Modeling. It admits
/// finite non-negative continuous diffusion and strictly negative drift while
/// avoiding an otherwise unnecessary overflow in the intermediate `2a`.
/// Unlike standardized maps in which `p` cancels algebraically, this function
/// actually returns `p`, so the stationary variance itself must be representable
/// as binary64. Callers decide whether zero stationary variance is admissible
/// for their named estimand.
pub(crate) fn recover_stationary_within_variance(
    continuous_diffusion: f64,
    log_rate: f64,
) -> Result<f64, LongitudinalError> {
    validate_stationary_process_inputs(continuous_diffusion, log_rate)?;
    if continuous_diffusion == 0.0 {
        return Ok(0.0);
    }

    let twice_rate = log_rate * 2.0;
    let stationary = if twice_rate.is_finite() {
        continuous_diffusion / -twice_rate
    } else {
        // In this branch |a| > MAX/2, so halving the finite numerator before
        // division is the exact power-of-two rescaling q/(2|a|). It avoids the
        // predecessor's division-then-quarter sequence, which could round twice
        // at the subnormal boundary. If q/2 itself underflows here, q/(2|a|)
        // is necessarily far below the minimum representable positive value.
        (continuous_diffusion * 0.5) / -log_rate
    };
    // Positive q with stable finite a implies strictly positive real p. If the
    // binary64 evaluation collapses that requested estimand to zero, zero is a
    // false scientific boundary rather than a representable stationary
    // variance. Standardized callers that algebraically cancel p use the input
    // validator above and therefore are not rejected by this materialization
    // rule.
    if !stationary.is_finite() || stationary == 0.0 {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }
    Ok(stationary)
}

#[cfg(test)]
mod tests {
    use super::{recover_stationary_within_variance, validate_stationary_process_inputs};
    use crate::LongitudinalError;

    #[test]
    fn stationary_input_admission_does_not_materialise_the_variance() {
        assert_eq!(
            validate_stationary_process_inputs(f64::from_bits(1), -1.0),
            Ok(())
        );
        assert_eq!(validate_stationary_process_inputs(f64::MAX, -0.25), Ok(()));
    }

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
    fn overflow_fallback_does_not_double_round_a_minimum_subnormal_result() {
        // The exact q / (-2a) rounds to the minimum positive subnormal. The
        // predecessor fallback rounded once during division and again during
        // its final quarter-scale, returning two subnormal ulps instead.
        let diffusion = f64::from_bits(0x3cdad6b3492a639e);
        let log_rate = -f64::from_bits(0x7fe2342c95642bec);
        let stationary = recover_stationary_within_variance(diffusion, log_rate)
            .expect("the final stationary variance is representable");
        assert_eq!(stationary.to_bits(), 1);
    }

    #[test]
    fn positive_stationary_variance_below_binary64_range_fails_closed() {
        let minimum_subnormal = f64::from_bits(1);
        assert_eq!(
            recover_stationary_within_variance(minimum_subnormal, -1.0),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
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
