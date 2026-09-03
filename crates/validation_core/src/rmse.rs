//! Root-mean-square error recovery metric.

use crate::ValidationError;
use crate::matching::absolute_residuals;
use crate::numeric::deterministic_compensated_sum;

struct ScaledRmse {
    scale: f64,
    rmse: f64,
    normalized_rmse: f64,
    normalized_mean_square: f64,
    normalized_squares: Vec<f64>,
}

fn scaled_rmse(residuals: &[f64]) -> Result<ScaledRmse, ValidationError> {
    if residuals.is_empty() || residuals.iter().any(|residual| !residual.is_finite()) {
        return Err(ValidationError::InvalidInput);
    }

    let scale = residuals
        .iter()
        .map(|residual| residual.abs())
        .fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(ScaledRmse {
            scale: 0.0,
            rmse: 0.0,
            normalized_rmse: 0.0,
            normalized_mean_square: 0.0,
            normalized_squares: vec![0.0; residuals.len()],
        });
    }

    let normalized_squares: Vec<_> = residuals
        .iter()
        .map(|residual| {
            let normalized = *residual / scale;
            normalized * normalized
        })
        .collect();
    let normalized_mean_square =
        deterministic_compensated_sum(normalized_squares.clone()) / residuals.len() as f64;
    let normalized_rmse = normalized_mean_square.sqrt();
    let rmse = scale * normalized_rmse;
    if !rmse.is_finite() || (rmse == 0.0 && normalized_rmse != 0.0) {
        Err(ValidationError::InvalidInput)
    } else {
        Ok(ScaledRmse {
            scale,
            rmse: if rmse == 0.0 { 0.0 } else { rmse },
            normalized_rmse,
            normalized_mean_square,
            normalized_squares,
        })
    }
}

/// Compute RMSE between truth and recovered parameter vectors.
///
/// Residuals are normalized by their largest magnitude before squaring. This
/// preserves a representable RMSE when raw residual squares would overflow or
/// underflow, while a mathematically non-zero RMSE that is itself below the
/// binary64 range fails closed rather than becoming false perfect recovery.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty, unequal-length,
/// non-finite inputs, an unrepresentable residual, or an unrepresentable final
/// RMSE.
pub fn root_mean_square_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    let residuals = absolute_residuals(truth, recovered)?;
    Ok(scaled_rmse(&residuals)?.rmse)
}

/// Approximate standard error of the RMSE under independent squared residuals.
///
/// Uses the delta-method form `se ≈ sd(r²) / (2 · RMSE · √n)` with sample SD of
/// squared residuals. Squared residuals stay normalized by the largest residual
/// magnitude, so representable RMSE standard errors do not fail because an
/// avoidable raw square or squared-deviation intermediate overflows. Returns
/// `0.0` when every residual is exactly zero or the squared residuals are
/// exactly constant.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for invalid pairs, when fewer than
/// two observations are present for a non-zero RMSE, or when a non-zero RMSE or
/// standard error is outside the representable binary64 range.
pub fn rmse_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    let residuals = absolute_residuals(truth, recovered)?;
    rmse_standard_error_from_residuals(&residuals)
}

#[inline(never)]
fn rmse_standard_error_from_residuals(residuals: &[f64]) -> Result<f64, ValidationError> {
    let scaled = scaled_rmse(residuals)?;
    if scaled.rmse == 0.0 {
        return Ok(0.0);
    }
    if residuals.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }

    let n = residuals.len() as f64;
    let normalized_deviation_squares: Vec<_> = scaled
        .normalized_squares
        .iter()
        .map(|value| {
            let deviation = *value - scaled.normalized_mean_square;
            deviation * deviation
        })
        .collect();
    let normalized_sample_variance =
        deterministic_compensated_sum(normalized_deviation_squares) / (n - 1.0);
    let denominator = 2.0 * scaled.normalized_rmse * n.sqrt();
    let normalized_standard_error = normalized_sample_variance.sqrt() / denominator;
    let standard_error = scaled.scale * normalized_standard_error;
    if !standard_error.is_finite()
        || (standard_error == 0.0 && normalized_standard_error != 0.0)
    {
        Err(ValidationError::InvalidInput)
    } else if standard_error == 0.0 {
        Ok(0.0)
    } else {
        Ok(standard_error)
    }
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
        assert_eq!(root_mean_square_error(&[1.0], &[1.0]), Ok(0.0));
        assert_eq!(rmse_standard_error(&[1.0], &[1.0]), Ok(0.0));
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
    }

    #[test]
    fn representable_extremes_avoid_raw_square_overflow() {
        assert_eq!(
            root_mean_square_error(&[0.0, 0.0], &[f64::MAX, f64::MAX]),
            Ok(f64::MAX)
        );
        assert_eq!(
            rmse_standard_error_from_residuals(&[f64::MAX, f64::MAX]),
            Ok(0.0)
        );

        let huge = 1e200;
        assert_eq!(
            rmse_standard_error_from_residuals(&[huge, huge, huge]),
            Ok(0.0)
        );

        let finite_se = rmse_standard_error_from_residuals(&[1e154, 0.0])
            .expect("normalized squared-residual deviations remain representable");
        assert!(finite_se.is_finite());
        assert!(finite_se > 0.0);
    }

    #[test]
    fn subnormal_rmse_distinguishes_representable_error_from_false_zero() {
        let ulp = f64::from_bits(1);
        assert_eq!(root_mean_square_error(&[0.0, 0.0], &[ulp, 0.0]), Ok(ulp));
        assert_eq!(
            root_mean_square_error(&[0.0, 0.0, 0.0, 0.0], &[ulp, 0.0, 0.0, 0.0]),
            Err(ValidationError::InvalidInput)
        );
    }

    #[test]
    fn unrepresentable_residual_still_fails_closed() {
        assert_eq!(
            root_mean_square_error(&[f64::MAX], &[-f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            rmse_standard_error(&[f64::MAX, 0.0], &[-f64::MAX, 0.0]),
            Err(ValidationError::InvalidInput)
        );
    }
}
