//! Parameter matching for truth-versus-recovered recovery studies.

use crate::ValidationError;
use crate::input::require_paired_finite;

/// Pairwise absolute residuals between truth and recovered parameters.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when lengths differ, inputs are
/// empty, any value is non-finite, or a finite input pair has an absolute
/// residual outside binary64 range.
pub fn absolute_residuals(truth: &[f64], recovered: &[f64]) -> Result<Vec<f64>, ValidationError> {
    require_paired_finite(truth, recovered)?;
    let mut residuals = Vec::with_capacity(truth.len());
    for (t, r) in truth.iter().zip(recovered) {
        let residual = (t - r).abs();
        if !residual.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        residuals.push(residual);
    }
    Ok(residuals)
}

/// Count exact matches within absolute tolerance `epsilon`.
///
/// The decision metric does not require every absolute residual to be
/// representable. If subtraction of two finite endpoints overflows, the true
/// absolute residual is larger than `f64::MAX` and therefore larger than every
/// admitted finite tolerance, so that pair is deterministically a mismatch.
/// `absolute_residuals` remains fail-closed when callers request the residual
/// magnitude itself.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for bad vectors or non-finite
/// `epsilon`, and [`ValidationError::InvalidConfiguration`] when `epsilon < 0`.
pub fn match_count(
    truth: &[f64],
    recovered: &[f64],
    epsilon: f64,
) -> Result<usize, ValidationError> {
    if !epsilon.is_finite() {
        return Err(ValidationError::InvalidInput);
    }
    if epsilon < 0.0 {
        return Err(ValidationError::InvalidConfiguration);
    }
    require_paired_finite(truth, recovered)?;
    Ok(truth
        .iter()
        .zip(recovered)
        .filter(|(truth_value, recovered_value)| {
            let residual = (**truth_value - **recovered_value).abs();
            residual.is_finite() && residual <= epsilon
        })
        .count())
}

#[cfg(test)]
mod tests {
    use super::{absolute_residuals, match_count};
    use crate::ValidationError;

    #[test]
    fn residuals_and_matches_are_oracle_correct() {
        let truth = [1.0, 2.0, 3.0];
        let recovered = [1.0, 2.1, 2.5];
        let residuals = absolute_residuals(&truth, &recovered).expect("ok");
        assert!((residuals[0] - 0.0).abs() < 1e-12);
        assert!((residuals[1] - 0.1).abs() < 1e-12);
        assert!((residuals[2] - 0.5).abs() < 1e-12);
        assert_eq!(match_count(&truth, &recovered, 0.11).expect("ok"), 2);
        assert_eq!(
            absolute_residuals(&[], &[]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            absolute_residuals(&[1.0], &[1.0, 2.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            absolute_residuals(&[f64::NAN], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            match_count(&truth, &recovered, -0.1),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            match_count(&truth, &recovered, f64::NAN),
            Err(ValidationError::InvalidInput)
        );
        // Opposite-sign extremes have no representable binary64 residual.
        assert_eq!(
            absolute_residuals(&[f64::MAX], &[-f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        // The threshold decision is still exact for every finite epsilon.
        assert_eq!(match_count(&[f64::MAX], &[-f64::MAX], f64::MAX), Ok(0));
    }
}
