//! Rubin total variance for draw-level OLS loadings.
//!
//! Rubin (1996, p. 473), restating Rubin (1987):
//! `T_m = Ū_m + (1 + 1/m) B_m`, where `Ū_m` is the mean complete-data
//! sampling variance and `B_m` is the between-draw variance of the point
//! estimates. This combines complete-data OLS loadings. It is not Mislevy
//! plausible-value draws.

use crate::error::PsychometricError;
use crate::indicator::{IndicatorKind, require_finite, require_valid_indicator};
use crate::loading::ordinary_least_squares_fit;

/// Rubin-combined OLS loading and total variance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RubinCombinedLoading {
    /// Mean complete-data loading `Q̄`.
    pub mean_loading: f64,
    /// Mean complete-data sampling variance `Ū`.
    pub within_variance: f64,
    /// Between-draw variance `B`.
    pub between_variance: f64,
    /// Total variance `T = Ū + (1 + 1/m) B`.
    pub total_variance: f64,
    /// Number of complete-data draws `m`.
    pub draw_count: usize,
}

/// Combine OLS loadings across posterior indicator draws with Rubin `T`.
///
/// Each draw contributes `Q̂_ℓ = λ_ℓ` and
/// `U_ℓ = σ̂²_ℓ / Σ (f − f̄)²`. The helper does not treat the draws as
/// Mislevy person-level plausible values.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when a draw is empty or
/// non-finite, [`PsychometricError::InsufficientDraws`] when fewer than two
/// draws are supplied, and indicator-kind or OLS errors from a draw.
pub fn combine_draw_level_ols_loadings(
    factor_scores: &[f64],
    indicator_draws: &[Vec<f64>],
    kind: IndicatorKind,
) -> Result<RubinCombinedLoading, PsychometricError> {
    require_valid_indicator(kind)?;
    if indicator_draws.len() < 2 {
        return Err(PsychometricError::InsufficientDraws);
    }
    let draw_count = indicator_draws.len();
    let mut loadings = Vec::with_capacity(draw_count);
    let mut within = Vec::with_capacity(draw_count);
    for draw in indicator_draws {
        let fit = ordinary_least_squares_fit(factor_scores, draw)?;
        if fit.predictor_sum_of_squares <= 0.0 {
            return Err(PsychometricError::SingularDesign);
        }
        let sampling_variance =
            require_finite(fit.residual_variance / fit.predictor_sum_of_squares)?;
        loadings.push(fit.slope);
        within.push(sampling_variance);
    }
    let count = draw_count as f64;
    let mut loading_sum = 0.0_f64;
    let mut within_sum = 0.0_f64;
    for index in 0..draw_count {
        loading_sum += loadings[index];
        within_sum += within[index];
    }
    let mean_loading = require_finite(loading_sum / count)?;
    let within_variance = require_finite(within_sum / count)?;
    let mut between_ss = 0.0_f64;
    for loading in &loadings {
        let deviation = loading - mean_loading;
        between_ss += deviation * deviation;
    }
    let between_variance = require_finite(between_ss / (count - 1.0))?;
    let total_variance = require_finite(within_variance + (1.0 + 1.0 / count) * between_variance)?;
    Ok(RubinCombinedLoading {
        mean_loading,
        within_variance,
        between_variance,
        total_variance,
        draw_count,
    })
}

#[cfg(test)]
mod tests {
    use super::combine_draw_level_ols_loadings;
    use crate::error::PsychometricError;
    use crate::indicator::IndicatorKind;

    #[test]
    fn rubin_t_matches_mean_plus_inflated_between() {
        let factors = [-1.0_f64, 0.0, 1.0];
        let draws = [vec![-0.7, 0.0, 0.7], vec![-0.9, 0.0, 0.9]];
        let combined =
            combine_draw_level_ols_loadings(&factors, &draws, IndicatorKind::AdditiveLogRatio)
                .expect("rubin");
        assert!((combined.mean_loading - 0.8).abs() < 1e-12);
        assert_eq!(combined.draw_count, 2);
        let expected_total =
            combined.within_variance + (1.0 + 1.0 / 2.0) * combined.between_variance;
        assert!((combined.total_variance - expected_total).abs() < 1e-15);
        assert!(combined.between_variance > 0.0);
        assert!(combined.within_variance.abs() < 1e-12);
    }

    #[test]
    fn raw_proportion_single_draw_and_bad_numeric_fail() {
        let factors = [0.0_f64, 1.0, 2.0];
        assert_eq!(
            combine_draw_level_ols_loadings(
                &factors,
                &[vec![0.0, 1.0, 2.0]],
                IndicatorKind::AdditiveLogRatio
            ),
            Err(PsychometricError::InsufficientDraws)
        );
        assert_eq!(
            combine_draw_level_ols_loadings(&factors, &[], IndicatorKind::AdditiveLogRatio),
            Err(PsychometricError::InsufficientDraws)
        );
        assert_eq!(
            combine_draw_level_ols_loadings(
                &factors,
                &[vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]],
                IndicatorKind::RawProportion
            ),
            Err(PsychometricError::RawProportionForbidden)
        );
        assert_eq!(
            combine_draw_level_ols_loadings(
                &factors,
                &[vec![0.0, f64::NAN, 2.0], vec![0.0, 1.0, 2.0]],
                IndicatorKind::IsometricLogRatio
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            combine_draw_level_ols_loadings(
                &[1.0, 1.0, 1.0],
                &[vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]],
                IndicatorKind::LogisticNormal
            ),
            Err(PsychometricError::SingularDesign)
        );
    }
}
