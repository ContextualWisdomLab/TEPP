//! Marginal normal interval construction from one location/covariance basis.
//!
//! Scientific recovery must not combine a truth-basis location with detached or
//! pre-alignment uncertainty. This module therefore validates the complete
//! covariance geometry before reading its diagonal to construct marginal normal
//! intervals. It remains coordinate arithmetic only; empirical calibration is a
//! separate recovery question.

use crate::ValidationError;

const COVARIANCE_SYMMETRY_RELATIVE_TOLERANCE: f64 = 1.0e-10;
const COVARIANCE_PSD_RELATIVE_TOLERANCE: f64 = 1.0e-12;

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
        let variance = covariance[index][index];
        if variance < 0.0 {
            return Err(ValidationError::InvalidInput);
        }
        let margin = z * variance.sqrt();
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

fn validate_covariance_matrix(
    covariance: &[Vec<f64>],
    coordinate_count: usize,
) -> Result<(), ValidationError> {
    if covariance.len() != coordinate_count
        || covariance.iter().any(|row| row.len() != coordinate_count)
        || covariance.iter().flatten().any(|value| !value.is_finite())
    {
        return Err(ValidationError::InvalidInput);
    }

    for row in 0..coordinate_count {
        if covariance[row][row] < 0.0 {
            return Err(ValidationError::InvalidInput);
        }
        for column in 0..row {
            let scale = covariance[row][column]
                .abs()
                .max(covariance[column][row].abs())
                .max(f64::EPSILON);
            let tolerance = COVARIANCE_SYMMETRY_RELATIVE_TOLERANCE * scale;
            if (covariance[row][column] - covariance[column][row]).abs() > tolerance {
                return Err(ValidationError::InvalidInput);
            }
        }
    }

    if !is_positive_semidefinite(covariance) {
        return Err(ValidationError::InvalidInput);
    }
    Ok(())
}

fn is_positive_semidefinite(covariance: &[Vec<f64>]) -> bool {
    let coordinate_count = covariance.len();
    let mut lower = vec![vec![0.0_f64; coordinate_count]; coordinate_count];

    for row in 0..coordinate_count {
        for column in 0..=row {
            let correction = (0..column)
                .map(|previous| lower[row][previous] * lower[column][previous])
                .sum::<f64>();
            if !correction.is_finite() {
                return false;
            }
            let residual = covariance[row][column] - correction;
            if !residual.is_finite() {
                return false;
            }
            let local_scale = covariance[row][column]
                .abs()
                .max(correction.abs())
                .max(f64::EPSILON);
            let tolerance = COVARIANCE_PSD_RELATIVE_TOLERANCE * local_scale;

            if row == column {
                if residual < -tolerance {
                    return false;
                }
                lower[row][column] = if residual <= tolerance {
                    0.0
                } else {
                    residual.sqrt()
                };
            } else if lower[column][column] > 0.0 {
                lower[row][column] = residual / lower[column][column];
                if !lower[row][column].is_finite() {
                    return false;
                }
            } else if residual.abs() > tolerance {
                return false;
            }
        }
    }
    true
}
