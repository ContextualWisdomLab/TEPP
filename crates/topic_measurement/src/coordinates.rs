//! Additive and isometric log-ratio maps for compositional topic coordinates.

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

/// Map a strictly positive unit simplex vector to isometric log-ratio coordinates.
///
/// The sequential Egozcue orthonormal basis sends a `K`-part composition to
/// the `K-1` vector whose Euclidean distance from another composition's ILR
/// vector equals their Aitchison distance. A vector norm is only the distance
/// from the equal-share origin. This is the coordinate system for distance-based
/// topic geometry. It is not the reference-dependent logistic-normal map; use
/// [`additive_log_ratio`] when that regression interface is the estimand.
///
/// # Errors
///
/// Returns [`TopicMeasurementError::InvalidComposition`] when the vector is
/// empty, has fewer than two parts, contains a non-finite or non-positive
/// entry, or does not sum to one within a tight absolute tolerance.
pub fn isometric_log_ratio(proportions: &[f64]) -> Result<Vec<f64>, TopicMeasurementError> {
    require_composition(proportions)?;
    let dimension = proportions.len();
    let logs: Vec<f64> = proportions.iter().map(|part| part.ln()).collect();
    let mut coordinates = Vec::with_capacity(dimension - 1);
    for index in 0..(dimension - 1) {
        let remaining = dimension - index - 1;
        #[allow(clippy::cast_precision_loss)]
        let remaining_f = remaining as f64;
        let scale = (remaining_f / (remaining_f + 1.0)).sqrt();
        let mut rest_sum = 0.0_f64;
        for log_part in &logs[index + 1..] {
            rest_sum += *log_part;
        }
        coordinates.push(scale * (logs[index] - rest_sum / remaining_f));
    }
    Ok(coordinates)
}

/// Invert isometric log-ratio coordinates back to the unit simplex.
///
/// # Errors
///
/// Returns [`TopicMeasurementError::InvalidLogRatioDimension`] when the
/// coordinate vector is empty, non-finite, or would underflow a part to zero
/// in the strictly positive `f64` simplex representation.
pub fn from_isometric_log_ratio(coordinates: &[f64]) -> Result<Vec<f64>, TopicMeasurementError> {
    if coordinates.is_empty() {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    for &value in coordinates {
        if !value.is_finite() {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
    }

    let dimension = coordinates.len() + 1;
    let mut centered_logs = vec![0.0_f64; dimension];
    for (index, &coordinate) in coordinates.iter().enumerate() {
        let remaining = dimension - index - 1;
        #[allow(clippy::cast_precision_loss)]
        let remaining_f = remaining as f64;
        let scale = (remaining_f / (remaining_f + 1.0)).sqrt();
        let negative = -1.0 / (remaining_f * (remaining_f + 1.0)).sqrt();
        centered_logs[index] += scale * coordinate;
        for centered in &mut centered_logs[index + 1..] {
            *centered += negative * coordinate;
        }
    }

    let mut maximum = centered_logs[0];
    for &value in &centered_logs[1..] {
        maximum = maximum.max(value);
    }
    if !maximum.is_finite() {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }

    let mut weights = Vec::with_capacity(dimension);
    let mut denominator = 0.0_f64;
    for &value in &centered_logs {
        let weight = (value - maximum).exp();
        if weight == 0.0 {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        denominator += weight;
        weights.push(weight);
    }
    Ok(weights.iter().map(|weight| weight / denominator).collect())
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
    use super::{
        additive_log_ratio, from_additive_log_ratio, from_isometric_log_ratio, isometric_log_ratio,
    };
    use crate::error::TopicMeasurementError;

    #[test]
    fn two_part_equal_shares_are_zero_and_unrepresentable_extremes_fail_closed() {
        let pair = additive_log_ratio(&[0.5, 0.5]).expect("pair");
        assert_eq!(pair.len(), 1);
        assert!(pair[0].abs() < 1e-15);
        let recovered = from_additive_log_ratio(&pair).expect("inverse");
        assert!((recovered[0] - 0.5).abs() < 1e-15);
        assert!((recovered[1] - 0.5).abs() < 1e-15);
        assert_eq!(
            from_additive_log_ratio(&[1.0e9]),
            Err(TopicMeasurementError::InvalidLogRatioDimension)
        );
        assert_eq!(
            additive_log_ratio(&[f64::MAX, f64::MAX]),
            Err(TopicMeasurementError::InvalidComposition),
            "overflowing finite parts must fail closed because compensated mass is non-finite"
        );
        let origin = isometric_log_ratio(&[0.5, 0.5]).expect("ilr origin");
        assert!(origin[0].abs() < 1e-15);
        let recovered_ilr = from_isometric_log_ratio(&origin).expect("ilr inverse");
        assert!((recovered_ilr[0] - 0.5).abs() < 1e-15);
        assert_eq!(
            from_isometric_log_ratio(&[1000.0]),
            Err(TopicMeasurementError::InvalidLogRatioDimension)
        );
        assert_eq!(
            from_isometric_log_ratio(&[-f64::MAX, f64::MAX]),
            Err(TopicMeasurementError::InvalidLogRatioDimension)
        );
        let three = isometric_log_ratio(&[2.0 / 6.0, 3.0 / 6.0, 1.0 / 6.0]).expect("ilr three");
        assert!((three[1] - (0.5_f64).sqrt() * 3.0_f64.ln()).abs() < 1e-15);
        let recovered_three = from_isometric_log_ratio(&three).expect("ilr three inverse");
        assert!((recovered_three.iter().sum::<f64>() - 1.0).abs() < 1e-15);
    }
}
