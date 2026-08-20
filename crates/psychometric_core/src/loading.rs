//! CPU `f64` ordinary-least-squares loading recovery.

use crate::error::PsychometricError;
use crate::indicator::{IndicatorKind, centered_pairs, require_finite, require_valid_indicator};

/// Ordinary least-squares intercept, slope, and residual variance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrdinaryLeastSquaresFit {
    /// Intercept `ν = ȳ − λ x̄`.
    pub intercept: f64,
    /// Slope `λ`.
    pub slope: f64,
    /// Residual variance `SSE / df` with `df = n − 2` when `n > 2`, else `0`.
    pub residual_variance: f64,
    /// Predictor sum of squared deviations `Σ (x − x̄)²`.
    pub predictor_sum_of_squares: f64,
}

/// Ordinary least-squares slope of `outcome` on `predictor`.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] for empty, singleton,
/// unequal-length, or non-finite vectors and
/// [`PsychometricError::SingularDesign`] when the predictor has zero variance.
pub fn ordinary_least_squares_slope(
    predictor: &[f64],
    outcome: &[f64],
) -> Result<f64, PsychometricError> {
    Ok(ordinary_least_squares_fit(predictor, outcome)?.slope)
}

/// Ordinary least-squares intercept and slope with residual variance.
///
/// Two-point lines have residual variance `0` because they fit exactly.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] for empty, singleton,
/// unequal-length, or non-finite vectors and
/// [`PsychometricError::SingularDesign`] when the predictor has zero variance.
pub fn ordinary_least_squares_fit(
    predictor: &[f64],
    outcome: &[f64],
) -> Result<OrdinaryLeastSquaresFit, PsychometricError> {
    let (pred_dev, out_dev, n) = centered_pairs(predictor, outcome)?;
    let mut cross = 0.0_f64;
    let mut pred_ss = 0.0_f64;
    for (pred, out) in pred_dev.iter().zip(&out_dev) {
        cross += pred * out;
        pred_ss += pred * pred;
    }
    if pred_ss <= 0.0 {
        return Err(PsychometricError::SingularDesign);
    }
    let slope = require_finite(cross / pred_ss)?;
    let mut outcome_sum = 0.0_f64;
    let mut predictor_sum = 0.0_f64;
    for (&pred, &out) in predictor.iter().zip(outcome) {
        predictor_sum += pred;
        outcome_sum += out;
    }
    let intercept = require_finite(outcome_sum / n - slope * (predictor_sum / n))?;
    let mut sse = 0.0_f64;
    for (&pred, &out) in pred_dev.iter().zip(&out_dev) {
        let residual = out - slope * pred;
        sse += residual * residual;
    }
    let residual_variance = if n > 2.0 {
        require_finite(sse / (n - 2.0))?
    } else {
        0.0
    };
    Ok(OrdinaryLeastSquaresFit {
        intercept,
        slope,
        residual_variance,
        predictor_sum_of_squares: pred_ss,
    })
}

/// Recover a single reflective loading from factor scores and an indicator.
///
/// # Errors
///
/// Returns the indicator-kind or OLS errors from
/// [`require_valid_indicator`] and [`ordinary_least_squares_slope`].
pub fn recover_reflective_loading(
    factor_scores: &[f64],
    indicators: &[f64],
    kind: IndicatorKind,
) -> Result<f64, PsychometricError> {
    require_valid_indicator(kind)?;
    ordinary_least_squares_slope(factor_scores, indicators)
}

#[cfg(test)]
mod tests {
    use super::{
        ordinary_least_squares_fit, ordinary_least_squares_slope, recover_reflective_loading,
    };
    use crate::error::PsychometricError;
    use crate::indicator::IndicatorKind;

    #[test]
    fn unit_slope_recovers_and_empty_or_overflow_input_fails() {
        let slope = ordinary_least_squares_slope(&[0.0, 1.0], &[0.0, 1.0]).expect("unit");
        assert!((slope - 1.0).abs() < 1e-15);
        let fit = ordinary_least_squares_fit(&[0.0, 1.0, 2.0], &[1.0, 3.0, 5.0]).expect("line");
        assert!((fit.slope - 2.0).abs() < 1e-12);
        assert!((fit.intercept - 1.0).abs() < 1e-12);
        assert!(fit.residual_variance.abs() < 1e-12);
        assert!(fit.predictor_sum_of_squares > 0.0);
        assert_eq!(
            recover_reflective_loading(&[], &[], IndicatorKind::AdditiveLogRatio),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            ordinary_least_squares_slope(&[0.0, f64::MAX], &[0.0, f64::MAX]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            ordinary_least_squares_fit(&[1.0, 1.0], &[2.0, 3.0]),
            Err(PsychometricError::SingularDesign)
        );
    }
}
