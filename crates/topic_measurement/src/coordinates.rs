//! Additive log-ratio maps for logistic-normal topic coordinates.

use crate::error::TopicMeasurementError;

const UNIT_SUM_TOLERANCE: f64 = 1e-12;

/// Map a strictly positive unit simplex vector to additive log-ratio coordinates.
///
/// For a `K`-part composition `θ` the image is the `K-1` vector
/// `y_k = ln(θ_k / θ_K)`. This is the logistic-normal coordinate system used
/// by correlated topic models and required before Euclidean or ESEM/DSEM work.
///
/// # Errors
///
/// Returns [`TopicMeasurementError::InvalidComposition`] when the vector is
/// empty, has fewer than two parts, contains a non-finite or non-positive
/// entry, or does not sum to one within a tight absolute tolerance.
pub fn additive_log_ratio(proportions: &[f64]) -> Result<Vec<f64>, TopicMeasurementError> {
    let last = require_composition(proportions)?;
    Ok(proportions[..proportions.len() - 1]
        .iter()
        .map(|part| (part / last).ln())
        .collect())
}

/// Invert additive log-ratio coordinates back to the unit simplex.
///
/// # Errors
///
/// Returns [`TopicMeasurementError::InvalidLogRatioDimension`] when the
/// coordinate vector is empty or contains a non-finite value.
pub fn from_additive_log_ratio(coordinates: &[f64]) -> Result<Vec<f64>, TopicMeasurementError> {
    if coordinates.is_empty() {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    let mut exponentiated = Vec::with_capacity(coordinates.len());
    let mut denom = 1.0_f64;
    for &value in coordinates {
        if !value.is_finite() {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        let exp = value.exp();
        if !exp.is_finite() {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        denom += exp;
        exponentiated.push(exp);
    }
    if !denom.is_finite() || denom <= 0.0 {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    let mut simplex = Vec::with_capacity(coordinates.len() + 1);
    for exp in exponentiated {
        simplex.push(exp / denom);
    }
    simplex.push(1.0 / denom);
    Ok(simplex)
}

fn require_composition(proportions: &[f64]) -> Result<f64, TopicMeasurementError> {
    if proportions.len() < 2 {
        return Err(TopicMeasurementError::InvalidComposition);
    }
    let mut sum = 0.0_f64;
    for &part in proportions {
        if !part.is_finite() || part <= 0.0 {
            return Err(TopicMeasurementError::InvalidComposition);
        }
        sum += part;
    }
    if !sum.is_finite() || (sum - 1.0).abs() > UNIT_SUM_TOLERANCE {
        return Err(TopicMeasurementError::InvalidComposition);
    }
    Ok(proportions[proportions.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::{additive_log_ratio, from_additive_log_ratio};
    use crate::error::TopicMeasurementError;

    #[test]
    fn two_part_equal_shares_are_zero_and_overflow_fails_closed() {
        let pair = additive_log_ratio(&[0.5, 0.5]).expect("pair");
        assert_eq!(pair.len(), 1);
        assert!(pair[0].abs() < 1e-15);
        assert_eq!(
            from_additive_log_ratio(&[1.0e9]),
            Err(TopicMeasurementError::InvalidLogRatioDimension)
        );
    }
}
