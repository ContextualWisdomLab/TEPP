//! Plausible-value aggregation of posterior structural draws.

use crate::error::PsychometricError;
use crate::indicator::{IndicatorKind, require_finite, require_valid_indicator};
use crate::loading::recover_reflective_loading;

/// Arithmetic mean of finite plausible-value draws.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when `draws` is empty or
/// contains a non-finite value.
pub fn plausible_value_mean(draws: &[f64]) -> Result<f64, PsychometricError> {
    if draws.is_empty() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut sum = 0.0_f64;
    for &value in draws {
        if !value.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        sum += value;
    }
    require_finite(sum / draws.len() as f64)
}

/// Recover a reflective loading by averaging OLS slopes across posterior
/// indicator draws (Rubin-style plausible values).
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when no draws are
/// supplied, and otherwise the first indicator-kind or OLS error from a draw.
pub fn recover_loading_from_plausible_values(
    factor_scores: &[f64],
    indicator_draws: &[Vec<f64>],
    kind: IndicatorKind,
) -> Result<f64, PsychometricError> {
    require_valid_indicator(kind)?;
    if indicator_draws.is_empty() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut recovered = Vec::with_capacity(indicator_draws.len());
    for draw in indicator_draws {
        recovered.push(recover_reflective_loading(factor_scores, draw, kind)?);
    }
    plausible_value_mean(&recovered)
}

#[cfg(test)]
mod tests {
    use super::{plausible_value_mean, recover_loading_from_plausible_values};
    use crate::error::PsychometricError;
    use crate::indicator::IndicatorKind;

    #[test]
    fn mean_of_two_draws_and_nonfinite_mean_fail_closed() {
        let mean = plausible_value_mean(&[1.0, 3.0]).expect("mean");
        assert!((mean - 2.0).abs() < 1e-15);
        assert_eq!(
            recover_loading_from_plausible_values(
                &[0.0, 1.0],
                &[vec![0.0, f64::NAN]],
                IndicatorKind::AdditiveLogRatio
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }
}
