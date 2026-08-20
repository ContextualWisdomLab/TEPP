//! Valid structural indicator coordinates and compositional-geometry claims.

use crate::error::PsychometricError;

/// Kind of indicator coordinates offered to a structural model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndicatorKind {
    /// Additive log-ratio (logistic-normal) coordinates.
    AdditiveLogRatio,
    /// Isometric log-ratio coordinates.
    IsometricLogRatio,
    /// Logistic-normal coordinates already mapped from the simplex.
    LogisticNormal,
    /// Raw topic proportions on the simplex.
    RawProportion,
}

impl IndicatorKind {
    /// Stable wire name for the indicator kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdditiveLogRatio => "alr",
            Self::IsometricLogRatio => "ilr",
            Self::LogisticNormal => "logistic_normal",
            Self::RawProportion => "raw_proportion",
        }
    }

    /// Return whether the kind is an admissible unconstrained structural input.
    ///
    /// This does not claim that the coordinates are orthonormal or preserve
    /// Aitchison distance. ALR is reference-dependent; only ILR carries that
    /// orthonormal compositional-geometry claim.
    #[must_use]
    pub const fn is_valid_structural_input(self) -> bool {
        !matches!(self, Self::RawProportion)
    }

    /// Return whether the coordinate kind is an orthonormal Aitchison isometry.
    #[must_use]
    pub const fn preserves_aitchison_distance(self) -> bool {
        matches!(self, Self::IsometricLogRatio)
    }
}

/// Refuse raw topic proportions as psychometric indicators.
///
/// # Errors
///
/// Returns [`PsychometricError::RawProportionForbidden`] for
/// [`IndicatorKind::RawProportion`].
pub fn require_valid_indicator(kind: IndicatorKind) -> Result<(), PsychometricError> {
    if kind.is_valid_structural_input() {
        Ok(())
    } else {
        Err(PsychometricError::RawProportionForbidden)
    }
}

/// Pearson product-moment correlation on already-mapped coordinates.
///
/// For ALR this is a reference-dependent coordinate correlation, not an
/// Aitchison-distance-preserving statistic. Use an ILR basis when orthonormal
/// compositional geometry is part of the estimand.
///
/// # Errors
///
/// Returns [`PsychometricError::RawProportionForbidden`] when `kind` is a raw
/// simplex, [`PsychometricError::InvalidNumericInput`] for empty, singleton,
/// unequal-length, or non-finite vectors, and
/// [`PsychometricError::SingularDesign`] when either vector has zero variance.
pub fn pearson_correlation(
    left: &[f64],
    right: &[f64],
    kind: IndicatorKind,
) -> Result<f64, PsychometricError> {
    require_valid_indicator(kind)?;
    let (left_dev, right_dev, _) = centered_pairs(left, right)?;
    let mut cross = 0.0_f64;
    let mut left_ss = 0.0_f64;
    let mut right_ss = 0.0_f64;
    for (left_value, right_value) in left_dev.iter().zip(&right_dev) {
        cross += left_value * right_value;
        left_ss += left_value * left_value;
        right_ss += right_value * right_value;
    }
    if left_ss <= 0.0 || right_ss <= 0.0 {
        return Err(PsychometricError::SingularDesign);
    }
    let denom = (left_ss * right_ss).sqrt();
    require_finite(cross / denom)
}

pub(crate) fn centered_pairs(
    left: &[f64],
    right: &[f64],
) -> Result<(Vec<f64>, Vec<f64>, f64), PsychometricError> {
    if left.len() < 2 || left.len() != right.len() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let n = left.len() as f64;
    let mut left_sum = 0.0_f64;
    let mut right_sum = 0.0_f64;
    for (left_value, right_value) in left.iter().zip(right) {
        if !left_value.is_finite() || !right_value.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        left_sum += left_value;
        right_sum += right_value;
    }
    let left_mean = left_sum / n;
    let right_mean = right_sum / n;
    let left_dev: Vec<f64> = left.iter().map(|value| value - left_mean).collect();
    let right_dev: Vec<f64> = right.iter().map(|value| value - right_mean).collect();
    Ok((left_dev, right_dev, n))
}

pub(crate) fn require_finite(value: f64) -> Result<f64, PsychometricError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PsychometricError::InvalidNumericInput)
    }
}

#[cfg(test)]
mod tests {
    use super::{IndicatorKind, pearson_correlation, require_valid_indicator};
    use crate::error::PsychometricError;

    #[test]
    fn valid_kinds_pass_and_zero_right_variance_is_singular() {
        require_valid_indicator(IndicatorKind::IsometricLogRatio).expect("ilr");
        assert_eq!(
            require_valid_indicator(IndicatorKind::RawProportion),
            Err(PsychometricError::RawProportionForbidden)
        );
        assert_eq!(
            pearson_correlation(&[1.0, 2.0], &[3.0, 3.0], IndicatorKind::LogisticNormal),
            Err(PsychometricError::SingularDesign)
        );
        assert_eq!(
            pearson_correlation(&[2.0, 2.0], &[1.0, 2.0], IndicatorKind::AdditiveLogRatio),
            Err(PsychometricError::SingularDesign)
        );
        assert_eq!(
            pearson_correlation(
                &[1.0, 2.0],
                &[1.0, f64::NAN],
                IndicatorKind::AdditiveLogRatio
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            pearson_correlation(
                &[0.0, f64::MAX],
                &[0.0, f64::MAX],
                IndicatorKind::AdditiveLogRatio
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }
}
