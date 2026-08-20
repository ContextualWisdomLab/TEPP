//! CPU `f64` ordinary-least-squares loading recovery.

use crate::error::PsychometricError;
use crate::indicator::{IndicatorKind, centered_pairs, require_finite, require_valid_indicator};

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
    let (pred_dev, out_dev, _) = centered_pairs(predictor, outcome)?;
    let mut cross = 0.0_f64;
    let mut pred_ss = 0.0_f64;
    for (pred, out) in pred_dev.iter().zip(&out_dev) {
        cross += pred * out;
        pred_ss += pred * pred;
    }
    if pred_ss <= 0.0 {
        return Err(PsychometricError::SingularDesign);
    }
    require_finite(cross / pred_ss)
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
    use super::{ordinary_least_squares_slope, recover_reflective_loading};
    use crate::error::PsychometricError;
    use crate::indicator::IndicatorKind;

    #[test]
    fn unit_slope_recovers_and_empty_or_overflow_input_fails() {
        let slope = ordinary_least_squares_slope(&[0.0, 1.0], &[0.0, 1.0]).expect("unit");
        assert!((slope - 1.0).abs() < 1e-15);
        assert_eq!(
            recover_reflective_loading(&[], &[], IndicatorKind::AdditiveLogRatio),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            ordinary_least_squares_slope(&[0.0, f64::MAX], &[0.0, f64::MAX]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            ordinary_least_squares_slope(&[1.0, 2.0], &[1.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            ordinary_least_squares_slope(&[f64::NAN, 2.0], &[1.0, 2.0]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            ordinary_least_squares_slope(&[1.0, 2.0], &[1.0, f64::NAN]),
            Err(PsychometricError::InvalidNumericInput)
        );
    }
}
