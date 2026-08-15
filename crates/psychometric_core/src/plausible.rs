//! Point-estimate aggregation across posterior structural draws.

use crate::error::PsychometricError;
use crate::indicator::{IndicatorKind, require_finite, require_valid_indicator};
use crate::loading::recover_reflective_loading;

/// Arithmetic mean of finite posterior-draw point estimates.
///
/// This helper does not pool within-draw and between-draw uncertainty and must
/// not be described as Rubin multiple-imputation variance pooling.
///
/// Scaling by the largest absolute draw prevents valid finite posterior values
/// from overflowing during aggregation. Compensated accumulation preserves
/// cancellation when draws span very different magnitudes.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when `draws` is empty or
/// contains a non-finite value.
pub fn posterior_draw_point_estimate_mean(draws: &[f64]) -> Result<f64, PsychometricError> {
    if draws.is_empty() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut scale = 0.0_f64;
    for &value in draws {
        if !value.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        scale = scale.max(value.abs());
    }
    if scale == 0.0 {
        return Ok(0.0);
    }

    let mut normalized_sum = 0.0_f64;
    let mut compensation = 0.0_f64;
    for &value in draws {
        let adjusted = value / scale - compensation;
        let next = normalized_sum + adjusted;
        compensation = (next - normalized_sum) - adjusted;
        normalized_sum = next;
    }
    require_finite((normalized_sum / draws.len() as f64) * scale)
}

/// Recover a reflective loading point estimate by averaging OLS slopes across
/// posterior indicator draws.
///
/// The result is a point-estimate summary only. It does not estimate within-draw
/// variance, between-draw variance, total variance, degrees of freedom, or a
/// confidence interval, and therefore is not Rubin-style uncertainty pooling.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when no draws are
/// supplied, and otherwise the first indicator-kind or OLS error from a draw.
pub fn recover_loading_point_estimate_mean(
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
    posterior_draw_point_estimate_mean(&recovered)
}

#[cfg(test)]
mod tests {
    use super::{posterior_draw_point_estimate_mean, recover_loading_point_estimate_mean};
    use crate::error::PsychometricError;
    use crate::indicator::IndicatorKind;

    #[test]
    fn mean_of_two_point_estimates_and_nonfinite_mean_fail_closed() {
        let mean = posterior_draw_point_estimate_mean(&[1.0, 3.0]).expect("mean");
        assert!((mean - 2.0).abs() < 1e-15);
        assert_eq!(
            recover_loading_point_estimate_mean(
                &[0.0, 1.0],
                &[vec![0.0, f64::NAN]],
                IndicatorKind::AdditiveLogRatio
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }
}
