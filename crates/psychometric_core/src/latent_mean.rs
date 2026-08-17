//! Two-group OLS strong/strict-gated latent-mean difference.
//!
//! Metric/weak invariance (equal loadings only) licenses shared *metric*
//! meaning. It does not license latent-mean comparison. Strong invariance
//! (equal loading and intercept) is required for means; strict additionally
//! equalizes residual variances. This is two-group OLS, not MGCFA.
//!
//! Wire names `configural` / `metric` / `scalar` match the unpublished
//! `measurement_invariance` crate (#84) without importing it. That crate's
//! `Metric` gate is not used here for latent means. `#84` `scalar` is the
//! strong/scalar status. Meredith (1993) names weak/strong/strict are used
//! only as conventional labels; that PDF was not opened.

use crate::error::PsychometricError;
use crate::indicator::{IndicatorKind, require_finite, require_valid_indicator};
use crate::loading::ordinary_least_squares_fit;

/// Two-group OLS invariance status for a mean comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MeanInvarianceStatus {
    /// Same regression form only. Loadings need not match.
    Configural,
    /// Equal loadings. `#84` wire name `metric`. Does not license means.
    Metric,
    /// Equal loadings and intercepts. `#84` wire name `scalar`.
    Strong,
    /// Strong plus equal residual variances.
    Strict,
}

impl MeanInvarianceStatus {
    /// Local status name (`strong` / `strict` keep Meredith's mean hierarchy).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Configural => "configural",
            Self::Metric => "metric",
            Self::Strong => "strong",
            Self::Strict => "strict",
        }
    }

    /// `#84` `measurement_invariance` wire name without importing that crate.
    #[must_use]
    pub const fn as_measurement_invariance_wire_name(self) -> &'static str {
        match self {
            Self::Configural => "configural",
            Self::Metric => "metric",
            Self::Strong => "scalar",
            Self::Strict => "strict",
        }
    }

    /// Return whether `#84` would license shared *metric* meaning.
    #[must_use]
    pub const fn licenses_shared_metric_meaning(self) -> bool {
        matches!(self, Self::Metric | Self::Strong | Self::Strict)
    }

    /// Return whether latent-mean comparison is licensed.
    #[must_use]
    pub const fn licenses_latent_mean_comparison(self) -> bool {
        matches!(self, Self::Strong | Self::Strict)
    }
}

/// One group's factor-score and indicator series.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupIndicatorSeries {
    /// Factor scores for the group.
    pub factor_scores: Vec<f64>,
    /// Indicator coordinates for the group.
    pub indicators: Vec<f64>,
}

/// Two-group OLS measurement parameters and status.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoGroupMeasurement {
    /// Reference-group intercept.
    pub reference_intercept: f64,
    /// Reference-group loading.
    pub reference_loading: f64,
    /// Comparison-group intercept.
    pub comparison_intercept: f64,
    /// Comparison-group loading.
    pub comparison_loading: f64,
    /// Reference residual variance.
    pub reference_residual_variance: f64,
    /// Comparison residual variance.
    pub comparison_residual_variance: f64,
    /// Classified invariance status.
    pub status: MeanInvarianceStatus,
}

/// Classify two-group OLS invariance from loadings, intercepts, and residuals.
///
/// # Errors
///
/// Returns indicator-kind or OLS errors from either group, and
/// [`PsychometricError::InvalidNumericInput`] when a tolerance is non-finite
/// or negative.
pub fn classify_two_group_ols_invariance(
    reference: &GroupIndicatorSeries,
    comparison: &GroupIndicatorSeries,
    kind: IndicatorKind,
    loading_tolerance: f64,
    intercept_tolerance: f64,
    residual_tolerance: f64,
) -> Result<TwoGroupMeasurement, PsychometricError> {
    require_valid_indicator(kind)?;
    if !loading_tolerance.is_finite()
        || loading_tolerance < 0.0
        || !intercept_tolerance.is_finite()
        || intercept_tolerance < 0.0
        || !residual_tolerance.is_finite()
        || residual_tolerance < 0.0
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let reference_fit =
        ordinary_least_squares_fit(&reference.factor_scores, &reference.indicators)?;
    let comparison_fit =
        ordinary_least_squares_fit(&comparison.factor_scores, &comparison.indicators)?;
    let loading_gap = (reference_fit.slope - comparison_fit.slope).abs();
    let intercept_gap = (reference_fit.intercept - comparison_fit.intercept).abs();
    let residual_gap = (reference_fit.residual_variance - comparison_fit.residual_variance).abs();
    let status = if loading_gap > loading_tolerance {
        MeanInvarianceStatus::Configural
    } else if intercept_gap > intercept_tolerance {
        MeanInvarianceStatus::Metric
    } else if residual_gap > residual_tolerance {
        MeanInvarianceStatus::Strong
    } else {
        MeanInvarianceStatus::Strict
    };
    Ok(TwoGroupMeasurement {
        reference_intercept: reference_fit.intercept,
        reference_loading: reference_fit.slope,
        comparison_intercept: comparison_fit.intercept,
        comparison_loading: comparison_fit.slope,
        reference_residual_variance: reference_fit.residual_variance,
        comparison_residual_variance: comparison_fit.residual_variance,
        status,
    })
}

/// Recover `(ȳ_c − ȳ_r) / λ` only under strong or strict invariance.
///
/// Metric/weak (equal loading, different intercept) fails closed.
///
/// # Errors
///
/// Returns [`PsychometricError::StrongInvarianceRequired`] when the classified
/// status is configural or metric, [`PsychometricError::SingularDesign`] when
/// the common loading is zero, and otherwise the classification errors.
pub fn recover_strong_gated_latent_mean_difference(
    reference: &GroupIndicatorSeries,
    comparison: &GroupIndicatorSeries,
    kind: IndicatorKind,
    loading_tolerance: f64,
    intercept_tolerance: f64,
    residual_tolerance: f64,
) -> Result<f64, PsychometricError> {
    let measurement = classify_two_group_ols_invariance(
        reference,
        comparison,
        kind,
        loading_tolerance,
        intercept_tolerance,
        residual_tolerance,
    )?;
    if !measurement.status.licenses_latent_mean_comparison() {
        return Err(PsychometricError::StrongInvarianceRequired);
    }
    let loading = require_finite(f64::midpoint(
        measurement.reference_loading,
        measurement.comparison_loading,
    ))?;
    if loading == 0.0 {
        return Err(PsychometricError::SingularDesign);
    }
    let reference_mean = series_mean(&reference.indicators)?;
    let comparison_mean = series_mean(&comparison.indicators)?;
    require_finite((comparison_mean - reference_mean) / loading)
}

fn series_mean(values: &[f64]) -> Result<f64, PsychometricError> {
    let mut sum = 0.0_f64;
    for &value in values {
        sum += value;
    }
    require_finite(sum / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        GroupIndicatorSeries, MeanInvarianceStatus, classify_two_group_ols_invariance,
        recover_strong_gated_latent_mean_difference,
    };
    use crate::error::PsychometricError;
    use crate::indicator::IndicatorKind;

    fn series(factors: &[f64], intercept: f64, loading: f64) -> GroupIndicatorSeries {
        GroupIndicatorSeries {
            factor_scores: factors.to_vec(),
            indicators: factors
                .iter()
                .map(|score| intercept + loading * score)
                .collect(),
        }
    }

    #[test]
    fn hash84_metric_wire_name_does_not_license_latent_means() {
        assert_eq!(
            MeanInvarianceStatus::Metric.as_measurement_invariance_wire_name(),
            "metric"
        );
        assert!(MeanInvarianceStatus::Metric.licenses_shared_metric_meaning());
        assert!(!MeanInvarianceStatus::Metric.licenses_latent_mean_comparison());
        assert_eq!(MeanInvarianceStatus::Metric.as_str(), "metric");
    }

    #[test]
    fn hash84_scalar_wire_name_is_strong_and_licenses_means() {
        assert_eq!(
            MeanInvarianceStatus::Strong.as_measurement_invariance_wire_name(),
            "scalar"
        );
        assert!(MeanInvarianceStatus::Strong.licenses_shared_metric_meaning());
        assert!(MeanInvarianceStatus::Strong.licenses_latent_mean_comparison());
        assert_eq!(MeanInvarianceStatus::Strong.as_str(), "strong");
        assert!(MeanInvarianceStatus::Strict.licenses_latent_mean_comparison());
        assert_eq!(
            MeanInvarianceStatus::Strict.as_measurement_invariance_wire_name(),
            "strict"
        );
        assert!(!MeanInvarianceStatus::Configural.licenses_shared_metric_meaning());
        assert!(!MeanInvarianceStatus::Configural.licenses_latent_mean_comparison());
        assert_eq!(
            MeanInvarianceStatus::Configural.as_measurement_invariance_wire_name(),
            "configural"
        );
    }

    #[test]
    fn strong_invariance_recovers_latent_mean_difference() {
        let reference = series(&[-1.0, 0.0, 1.0], 0.5, 1.2);
        let comparison = series(&[1.0, 2.0, 3.0], 0.5, 1.2);
        let difference = recover_strong_gated_latent_mean_difference(
            &reference,
            &comparison,
            IndicatorKind::AdditiveLogRatio,
            1e-9,
            1e-9,
            1e-9,
        )
        .expect("strong");
        // ȳ_r = 0.5, ȳ_c = 0.5+1.2*2 = 2.9, diff/λ = 2.4/1.2 = 2.0
        assert!((difference - 2.0).abs() < 1e-12);
        let classified = classify_two_group_ols_invariance(
            &reference,
            &comparison,
            IndicatorKind::AdditiveLogRatio,
            1e-9,
            1e-9,
            1e-9,
        )
        .expect("class");
        assert_eq!(classified.status, MeanInvarianceStatus::Strict);
    }

    #[test]
    fn metric_only_and_configural_refuse_latent_means() {
        let reference = series(&[-1.0, 0.0, 1.0], 0.5, 1.2);
        let metric_only = series(&[1.0, 2.0, 3.0], 1.5, 1.2);
        assert_eq!(
            recover_strong_gated_latent_mean_difference(
                &reference,
                &metric_only,
                IndicatorKind::AdditiveLogRatio,
                1e-9,
                1e-9,
                1e-9,
            ),
            Err(PsychometricError::StrongInvarianceRequired)
        );
        let classified = classify_two_group_ols_invariance(
            &reference,
            &metric_only,
            IndicatorKind::AdditiveLogRatio,
            1e-9,
            1e-9,
            1e-9,
        )
        .expect("metric");
        assert_eq!(classified.status, MeanInvarianceStatus::Metric);

        let configural = series(&[1.0, 2.0, 3.0], 0.5, 0.4);
        assert_eq!(
            recover_strong_gated_latent_mean_difference(
                &reference,
                &configural,
                IndicatorKind::AdditiveLogRatio,
                1e-9,
                1e-9,
                1e-9,
            ),
            Err(PsychometricError::StrongInvarianceRequired)
        );
        let classified = classify_two_group_ols_invariance(
            &reference,
            &configural,
            IndicatorKind::AdditiveLogRatio,
            1e-9,
            1e-9,
            1e-9,
        )
        .expect("configural");
        assert_eq!(classified.status, MeanInvarianceStatus::Configural);
    }

    #[test]
    fn strong_but_not_strict_still_licenses_means() {
        let reference = GroupIndicatorSeries {
            factor_scores: vec![-2.0, -1.0, 0.0, 1.0, 2.0],
            indicators: vec![0.5 - 2.4, 0.5 - 1.2, 0.5, 0.5 + 1.2, 0.5 + 2.4],
        };
        let comparison = GroupIndicatorSeries {
            factor_scores: vec![-2.0, -1.0, 0.0, 1.0, 2.0],
            indicators: vec![
                0.5 - 2.4 + 0.2,
                0.5 - 1.2 - 0.4,
                0.5,
                0.5 + 1.2 + 0.4,
                0.5 + 2.4 - 0.2,
            ],
        };
        let classified = classify_two_group_ols_invariance(
            &reference,
            &comparison,
            IndicatorKind::LogisticNormal,
            0.05,
            0.2,
            1e-12,
        )
        .expect("strong");
        assert_eq!(classified.status, MeanInvarianceStatus::Strong);
        let difference = recover_strong_gated_latent_mean_difference(
            &reference,
            &comparison,
            IndicatorKind::LogisticNormal,
            0.05,
            0.2,
            1e-12,
        )
        .expect("licensed");
        assert!(difference.is_finite());
    }

    #[test]
    fn invalid_tolerance_raw_kind_and_zero_loading_fail() {
        let reference = series(&[-1.0, 0.0, 1.0], 0.0, 1.0);
        let comparison = series(&[0.0, 1.0, 2.0], 0.0, 1.0);
        assert_eq!(
            classify_two_group_ols_invariance(
                &reference,
                &comparison,
                IndicatorKind::AdditiveLogRatio,
                f64::NAN,
                1e-9,
                1e-9,
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            classify_two_group_ols_invariance(
                &reference,
                &comparison,
                IndicatorKind::AdditiveLogRatio,
                -0.1,
                1e-9,
                1e-9,
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            classify_two_group_ols_invariance(
                &reference,
                &comparison,
                IndicatorKind::AdditiveLogRatio,
                1e-9,
                f64::INFINITY,
                1e-9,
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            classify_two_group_ols_invariance(
                &reference,
                &comparison,
                IndicatorKind::AdditiveLogRatio,
                1e-9,
                -0.01,
                1e-9,
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            classify_two_group_ols_invariance(
                &reference,
                &comparison,
                IndicatorKind::AdditiveLogRatio,
                1e-9,
                1e-9,
                f64::NAN,
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            classify_two_group_ols_invariance(
                &reference,
                &comparison,
                IndicatorKind::AdditiveLogRatio,
                1e-9,
                1e-9,
                -1.0,
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_strong_gated_latent_mean_difference(
                &reference,
                &comparison,
                IndicatorKind::RawProportion,
                1e-9,
                1e-9,
                1e-9,
            ),
            Err(PsychometricError::RawProportionForbidden)
        );
        let zero = series(&[-1.0, 0.0, 1.0], 2.0, 0.0);
        let other = series(&[0.0, 1.0, 2.0], 2.0, 0.0);
        assert_eq!(
            recover_strong_gated_latent_mean_difference(
                &zero,
                &other,
                IndicatorKind::IsometricLogRatio,
                1e-9,
                1e-9,
                1e-9,
            ),
            Err(PsychometricError::SingularDesign)
        );
    }
}
