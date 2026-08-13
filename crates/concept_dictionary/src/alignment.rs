//! CPU `f64` concept-coordinate alignment and invariance gates.

use crate::error::ConceptError;

/// Measurement-invariance evidence for a cross-language comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InvarianceLevel {
    /// No invariance evidence.
    None,
    /// Configural invariance only.
    Configural,
    /// Metric invariance.
    Metric,
    /// Scalar invariance.
    Scalar,
    /// Partial invariance sufficient for the claimed comparison.
    Partial,
}

impl InvarianceLevel {
    /// Stable wire name for the invariance level.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Configural => "configural",
            Self::Metric => "metric",
            Self::Scalar => "scalar",
            Self::Partial => "partial",
        }
    }

    /// Return whether latent-mean comparison is admissible.
    #[must_use]
    pub const fn admits_mean_comparison(self) -> bool {
        matches!(self, Self::Scalar | Self::Partial)
    }
}

/// Permit a cross-language mean comparison only with scalar or partial
/// invariance evidence (Meredith, 1993; Vandenberg & Lance, 2000).
///
/// # Errors
///
/// Returns [`ConceptError::InvarianceRequired`] when the invariance level is
/// insufficient for a mean comparison.
pub const fn compare_cross_language_means(level: InvarianceLevel) -> Result<(), ConceptError> {
    if level.admits_mean_comparison() {
        Ok(())
    } else {
        Err(ConceptError::InvarianceRequired)
    }
}

/// Root-mean-square error between two concept-coordinate vectors on CPU `f64`.
///
/// # Errors
///
/// Returns [`ConceptError::InvalidNumericInput`] for empty, unequal-length, or
/// non-finite vectors.
#[allow(clippy::cast_precision_loss)]
pub fn concept_coordinate_rmse(left: &[f64], right: &[f64]) -> Result<f64, ConceptError> {
    if left.is_empty() || left.len() != right.len() {
        return Err(ConceptError::InvalidNumericInput);
    }
    let mut sum_sq = 0.0_f64;
    for (left_value, right_value) in left.iter().zip(right) {
        if !left_value.is_finite() || !right_value.is_finite() {
            return Err(ConceptError::InvalidNumericInput);
        }
        let residual = left_value - right_value;
        sum_sq += residual * residual;
    }
    Ok((sum_sq / left.len() as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::{InvarianceLevel, compare_cross_language_means, concept_coordinate_rmse};
    use crate::error::ConceptError;

    #[test]
    fn invariance_wire_names_and_identity_rmse() {
        assert_eq!(InvarianceLevel::None.as_str(), "none");
        assert_eq!(InvarianceLevel::Configural.as_str(), "configural");
        assert_eq!(InvarianceLevel::Metric.as_str(), "metric");
        assert_eq!(InvarianceLevel::Scalar.as_str(), "scalar");
        assert_eq!(InvarianceLevel::Partial.as_str(), "partial");
        compare_cross_language_means(InvarianceLevel::Scalar).expect("scalar");
        assert_eq!(
            compare_cross_language_means(InvarianceLevel::Metric),
            Err(ConceptError::InvarianceRequired)
        );
        let error = concept_coordinate_rmse(&[1.0, 2.0], &[1.0, 2.0]).expect("identity");
        assert!(error < 1e-12);
    }
}
