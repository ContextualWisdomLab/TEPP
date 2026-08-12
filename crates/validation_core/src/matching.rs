//! Parameter matching for truth-versus-recovered recovery studies.

use crate::ValidationError;
use crate::input::require_paired_finite;

/// Pairwise absolute residuals between truth and recovered parameters.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when lengths differ, inputs are
/// empty, or any value is non-finite.
pub fn absolute_residuals(truth: &[f64], recovered: &[f64]) -> Result<Vec<f64>, ValidationError> {
    require_paired_finite(truth, recovered)?;
    Ok(truth
        .iter()
        .zip(recovered)
        .map(|(t, r)| (t - r).abs())
        .collect())
}

/// Count exact matches within absolute tolerance `epsilon`.
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
    let residuals = absolute_residuals(truth, recovered)?;
    Ok(residuals
        .iter()
        .filter(|residual| **residual <= epsilon)
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
    }
}
