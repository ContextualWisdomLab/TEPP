//! Marginal normal interval construction from one location/covariance basis.
//!
//! Scientific recovery must not combine a truth-basis location with detached or
//! pre-alignment uncertainty. This module therefore validates the complete
//! covariance geometry before reading its diagonal to construct marginal normal
//! intervals. It remains coordinate arithmetic only; empirical calibration is a
//! separate recovery question.

use crate::ValidationError;
use crate::topic_alignment::validate_covariance_matrix;

/// Closed marginal interval bounds in the same coordinate order as their location.
#[derive(Clone, Debug, PartialEq)]
pub struct MarginalIntervalBounds {
    lower: Vec<f64>,
    upper: Vec<f64>,
}

impl MarginalIntervalBounds {
    /// Return inclusive lower bounds in location-coordinate order.
    #[must_use]
    pub fn lower(&self) -> &[f64] {
        &self.lower
    }

    /// Return inclusive upper bounds in location-coordinate order.
    #[must_use]
    pub fn upper(&self) -> &[f64] {
        &self.upper
    }
}

/// Construct marginal normal/Wald intervals from one location/covariance basis.
///
/// For location `mu`, covariance `Sigma`, and positive normal critical value `z`,
/// each closed marginal interval is `mu_i ± z * sqrt(Sigma_ii)`. The complete
/// covariance matrix is validated as finite, symmetric, and positive
/// semidefinite before any diagonal is consumed, preventing malformed or
/// detached uncertainty geometry from looking like valid marginal evidence.
/// Zero marginal variance is allowed and produces a degenerate closed interval.
///
/// This function does not establish posterior calibration, independence among
/// coordinates/documents, semantic topic identity, or release authority. Those
/// claims require repeated scientific recovery over owner-admitted experiments.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidConfiguration`] when `z` is non-finite or
/// non-positive. Returns [`ValidationError::InvalidInput`] for empty/non-finite
/// locations, dimension mismatch, non-finite/asymmetric/indefinite covariance,
/// negative marginal variance, or non-finite interval arithmetic.
pub fn normal_marginal_interval_bounds(
    location: &[f64],
    covariance: &[Vec<f64>],
    z: f64,
) -> Result<MarginalIntervalBounds, ValidationError> {
    if !z.is_finite() || z <= 0.0 {
        return Err(ValidationError::InvalidConfiguration);
    }
    if location.is_empty() || location.iter().any(|value| !value.is_finite()) {
        return Err(ValidationError::InvalidInput);
    }
    validate_covariance_matrix(covariance, location.len())?;

    let mut lower = Vec::with_capacity(location.len());
    let mut upper = Vec::with_capacity(location.len());
    for (index, center) in location.iter().copied().enumerate() {
        let margin = z * covariance[index][index].sqrt();
        let low = center - margin;
        let high = center + margin;
        if !margin.is_finite() || !low.is_finite() || !high.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        lower.push(low);
        upper.push(high);
    }

    Ok(MarginalIntervalBounds { lower, upper })
}
