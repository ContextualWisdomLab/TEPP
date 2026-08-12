//! Root-mean-square error recovery metric.

use crate::ValidationError;
use crate::input::require_finite;
use crate::matching::absolute_residuals;

/// Compute RMSE between truth and recovered parameter vectors.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty, unequal-length,
/// non-finite inputs, or squared-residual overflow.
pub fn root_mean_square_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    let residuals = absolute_residuals(truth, recovered)?;
    let mut square_sum = 0.0_f64;
    for residual in &residuals {
        let square = residual * residual;
        if !square.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        square_sum += square;
    }
    require_finite((square_sum / residuals.len() as f64).sqrt())
}

/// Approximate standard error of the RMSE under independent squared residuals.
///
/// Uses the delta-method form `se ≈ sd(r²) / (2 · RMSE · √n)` with sample SD of
/// squared residuals. Returns `0.0` when RMSE is zero.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for invalid pairs or when fewer
/// than two observations are present for a non-zero RMSE.
pub fn rmse_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    let residuals = absolute_residuals(truth, recovered)?;
    rmse_standard_error_from_residuals(&residuals)
}

#[inline(never)]
fn rmse_standard_error_from_residuals(residuals: &[f64]) -> Result<f64, ValidationError> {
    let n = residuals.len() as f64;
    let mut square_sum = 0.0_f64;
    let mut squares = Vec::with_capacity(residuals.len());
    for residual in residuals {
        let square = residual * residual;
        if !square.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        squares.push(square);
        square_sum += square;
    }
    let rmse = require_finite((square_sum / n).sqrt())?;
    if rmse <= 0.0 {
        return Ok(0.0);
    }
    if residuals.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    let mean = require_finite(squares.iter().sum::<f64>() / n)?;
    let mut variance_sum = 0.0_f64;
    for value in &squares {
        let delta = value - mean;
        let square = delta * delta;
        if !square.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        variance_sum += square;
    }
    let variance = variance_sum / (n - 1.0);
    require_finite(require_finite(variance.sqrt())? / (2.0 * rmse * n.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::{rmse_standard_error, rmse_standard_error_from_residuals, root_mean_square_error};
    use crate::ValidationError;

    #[test]
    fn rmse_matches_oracle_and_zero_recovery() {
        let truth = [0.0, 0.0, 0.0];
        let recovered = [3.0, 4.0, 0.0];
        let rmse = root_mean_square_error(&truth, &recovered).expect("ok");
        assert!((rmse - (25.0_f64 / 3.0).sqrt()).abs() < 1e-12);
        assert!((root_mean_square_error(&[1.0], &[1.0]).expect("zero") - 0.0).abs() < 1e-12);
        assert!((rmse_standard_error(&[1.0], &[1.0]).expect("se0") - 0.0).abs() < 1e-12);
        let se = rmse_standard_error(&truth, &recovered).expect("se");
        assert!(se.is_finite());
        assert!(se > 0.0);
        assert_eq!(
            rmse_standard_error(&[1.0], &[2.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            root_mean_square_error(&[], &[]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            rmse_standard_error_from_residuals(&[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(rmse_standard_error_from_residuals(&[0.0]), Ok(0.0));
        // Squared residual overflow yields non-finite RMSE.
        assert_eq!(
            rmse_standard_error_from_residuals(&[f64::MAX, f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            root_mean_square_error(&[0.0], &[f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
    }

    #[test]
    fn overflow_and_nonfinite_intermediates_fail_closed() {
        assert_eq!(
            root_mean_square_error(&[0.0, 0.0], &[f64::MAX, f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            rmse_standard_error_from_residuals(&[f64::MAX, f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        let huge = 1e200;
        assert_eq!(
            rmse_standard_error_from_residuals(&[huge, -huge, huge]),
            Err(ValidationError::InvalidInput)
        );
        // Finite residual squares whose variance deviations overflow.
        assert_eq!(
            rmse_standard_error_from_residuals(&[1e154, 0.0]),
            Err(ValidationError::InvalidInput)
        );
    }
}
