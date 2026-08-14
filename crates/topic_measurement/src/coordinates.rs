//! Additive log-ratio maps for logistic-normal topic coordinates.

use crate::error::TopicMeasurementError;

const UNIT_SUM_TOLERANCE: f64 = 1e-12;

/// Map a strictly positive unit simplex vector to additive log-ratio coordinates.
///
/// For a `K`-part composition `θ` the image is the `K-1` vector
/// `y_k = ln(θ_k / θ_K)`. This reference-dependent, full-rank coordinate map
/// supports logistic-normal regression and ESEM/DSEM interfaces. It is not an
/// orthonormal isometry for Aitchison distance; use ILR coordinates when that
/// Euclidean geometry is the estimand.
///
/// # Errors
///
/// Returns [`TopicMeasurementError::InvalidComposition`] when the vector is
/// empty, has fewer than two parts, contains a non-finite or non-positive
/// entry, or does not sum to one within a tight absolute tolerance.
pub fn additive_log_ratio(proportions: &[f64]) -> Result<Vec<f64>, TopicMeasurementError> {
    let last = require_composition(proportions)?;
    let reference_log = last.ln();
    Ok(proportions[..proportions.len() - 1]
        .iter()
        .map(|part| part.ln() - reference_log)
        .collect())
}

/// Invert additive log-ratio coordinates back to the unit simplex.
///
/// # Errors
///
/// Returns [`TopicMeasurementError::InvalidLogRatioDimension`] when the
/// coordinate vector is empty, non-finite, or would underflow a part to zero
/// in the strictly positive `f64` simplex representation.
pub fn from_additive_log_ratio(coordinates: &[f64]) -> Result<Vec<f64>, TopicMeasurementError> {
    if coordinates.is_empty() {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    let mut maximum = 0.0_f64;
    for &value in coordinates {
        if !value.is_finite() {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        maximum = maximum.max(value);
    }

    let reference_weight = (-maximum).exp();
    if reference_weight == 0.0 {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    let mut shifted_weights = Vec::with_capacity(coordinates.len());
    let mut denominator = reference_weight;
    for &value in coordinates {
        let weight = (value - maximum).exp();
        if weight == 0.0 {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        denominator += weight;
        shifted_weights.push(weight);
    }

    let mut simplex = Vec::with_capacity(coordinates.len() + 1);
    for weight in shifted_weights {
        simplex.push(weight / denominator);
    }
    simplex.push(reference_weight / denominator);
    Ok(simplex)
}

fn require_composition(proportions: &[f64]) -> Result<f64, TopicMeasurementError> {
    if proportions.len() < 2 {
        return Err(TopicMeasurementError::InvalidComposition);
    }
    let mut sum = 0.0_f64;
    let mut compensation = 0.0_f64;
    for &part in proportions {
        if !part.is_finite() || part <= 0.0 {
            return Err(TopicMeasurementError::InvalidComposition);
        }
        let next = sum + part;
        compensation += if sum.abs() >= part.abs() {
            (sum - next) + part
        } else {
            (part - next) + sum
        };
        sum = next;
    }
    let compensated_sum = sum + compensation;
    if !compensated_sum.is_finite() || (compensated_sum - 1.0).abs() > UNIT_SUM_TOLERANCE {
        return Err(TopicMeasurementError::InvalidComposition);
    }
    Ok(proportions[proportions.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::{additive_log_ratio, from_additive_log_ratio};
    use crate::error::TopicMeasurementError;

    #[test]
    fn two_part_equal_shares_are_zero_and_unrepresentable_extremes_fail_closed() {
        let pair = additive_log_ratio(&[0.5, 0.5]).expect("pair");
        assert_eq!(pair.len(), 1);
        assert!(pair[0].abs() < 1e-15);
        assert_eq!(
            from_additive_log_ratio(&[1.0e9]),
            Err(TopicMeasurementError::InvalidLogRatioDimension)
        );
    }
}
