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
    let mut shifted_weights = Vec::with_capacity(coordinates.len() + 1);
    for &value in coordinates {
        let weight = (value - maximum).exp();
        if weight == 0.0 {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        shifted_weights.push(weight);
    }
    shifted_weights.push(reference_weight);
    normalize_positive_weights(shifted_weights)
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
        let mut rest_compensation = 0.0_f64;
        for &log_part in &logs[index + 1..] {
            let next = rest_sum + log_part;
            rest_compensation += if rest_sum.abs() >= log_part.abs() {
                (rest_sum - next) + log_part
            } else {
                (log_part - next) + rest_sum
            };
            rest_sum = next;
        }
        let compensated_rest = rest_sum + rest_compensation;
        coordinates.push(scale * (logs[index] - compensated_rest / remaining_f));
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
    for &value in &centered_logs {
        let weight = (value - maximum).exp();
        if weight == 0.0 {
            return Err(TopicMeasurementError::InvalidLogRatioDimension);
        }
        weights.push(weight);
    }
    normalize_positive_weights(weights)
}

fn normalize_positive_weights(weights: Vec<f64>) -> Result<Vec<f64>, TopicMeasurementError> {
    // Retain small positive mass before dividing, as simplex admission does.
    let mut sum = 0.0_f64;
    let mut compensation = 0.0_f64;
    for &weight in &weights {
        let next = sum + weight;
        compensation += if sum.abs() >= weight.abs() {
            (sum - next) + weight
        } else {
            (weight - next) + sum
        };
        sum = next;
    }
    let denominator = sum + compensation;
    let simplex: Vec<f64> = weights
        .into_iter()
        .map(|weight| weight / denominator)
        .collect();
    if simplex.iter().any(|part| *part <= 0.0) {
        return Err(TopicMeasurementError::InvalidLogRatioDimension);
    }
    Ok(simplex)
}

/// Aitchison distance between two strictly positive unit simplex vectors.
///
/// The distance is the Euclidean norm of the clr residual
/// `clr(x) − clr(y)`. Sequential ILR is an isometry for this distance, so
/// `‖ilr(x) − ilr(y)‖` must recover the same value. A single ILR vector
/// norm is only the distance from the equal-share origin.
///
/// # Errors
///
/// Returns [`TopicMeasurementError::InvalidComposition`] when the vectors
/// have unequal length or fail simplex validation.
pub fn aitchison_distance(left: &[f64], right: &[f64]) -> Result<f64, TopicMeasurementError> {
    if left.len() != right.len() {
        return Err(TopicMeasurementError::InvalidComposition);
    }
    require_composition(left)?;
    require_composition(right)?;
    #[allow(clippy::cast_precision_loss)]
    let parts = left.len() as f64;
    let mut left_log_sum = 0.0_f64;
    let mut right_log_sum = 0.0_f64;
    let mut left_compensation = 0.0_f64;
    let mut right_compensation = 0.0_f64;
    for index in 0..left.len() {
        let left_log = left[index].ln();
        let right_log = right[index].ln();
        let left_next = left_log_sum + left_log;
        left_compensation += if left_log_sum.abs() >= left_log.abs() {
            (left_log_sum - left_next) + left_log
        } else {
            (left_log - left_next) + left_log_sum
        };
        left_log_sum = left_next;
        let right_next = right_log_sum + right_log;
        right_compensation += if right_log_sum.abs() >= right_log.abs() {
            (right_log_sum - right_next) + right_log
        } else {
            (right_log - right_next) + right_log_sum
        };
        right_log_sum = right_next;
    }
    left_log_sum += left_compensation;
    right_log_sum += right_compensation;
    let left_mean = left_log_sum / parts;
    let right_mean = right_log_sum / parts;
    let mut square_sum = 0.0_f64;
    let mut square_compensation = 0.0_f64;
    for index in 0..left.len() {
        let residual = (left[index].ln() - left_mean) - (right[index].ln() - right_mean);
        let term = residual * residual;
        let next = square_sum + term;
        square_compensation += if square_sum.abs() >= term.abs() {
            (square_sum - next) + term
        } else {
            (term - next) + square_sum
        };
        square_sum = next;
    }
    Ok((square_sum + square_compensation).sqrt())
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
        additive_log_ratio, aitchison_distance, from_additive_log_ratio, from_isometric_log_ratio,
        isometric_log_ratio,
    };
    use crate::error::TopicMeasurementError;

    #[test]
    fn relative_equal_share_shift_keeps_aitchison_distance_at_the_origin() {
        // Equal-share compositions are one point in Aitchison geometry, so the clr
        // distance and every ILR coordinate difference vanish in exact arithmetic.
        // Each ILR coordinate subtracts a compensated mean of identical logarithms,
        // so its rounding error is a small multiple of eps·|ln share| at every part
        // count (Ogita et al., 2005, eq. 4.8). A distance adds one such error per
        // coordinate in quadrature, so its bound scales with the square root of the
        // coordinate count. Offsets are ULPs from the exact share; +450 is the
        // published `share * (1 + 1e-13)` literal and the others are admitted
        // neighbours whose distance exceeded the unscaled bound.
        let cases: [(usize, i64); 6] = [
            (512, 450),
            (512, 91),
            (512, 603),
            (16_384, 450),
            (16_384, -5_363),
            (16_384, 4_096),
        ];
        for (parts, offset) in cases {
            #[allow(clippy::cast_precision_loss)]
            let parts_f = parts as f64;
            let share = 1.0_f64 / parts_f;
            let right_share = f64::from_bits(
                share
                    .to_bits()
                    .checked_add_signed(offset)
                    .expect("finite offset"),
            );
            if offset == 450 {
                assert_eq!(right_share.to_bits(), (share * (1.0 + 1e-13)).to_bits());
            }
            let left = vec![share; parts];
            let right = vec![right_share; parts];
            let bound = 8.0 * f64::EPSILON * share.ln().abs();
            let distance = aitchison_distance(&left, &right).expect("aitchison");
            assert!(
                distance <= bound * parts_f.sqrt(),
                "{parts} parts at {offset} ulp: clr distance {distance} exceeds sqrt(D)*{bound}"
            );
            let left_ilr = isometric_log_ratio(&left).expect("left ilr");
            let right_ilr = isometric_log_ratio(&right).expect("right ilr");
            assert_eq!(left_ilr.len(), parts - 1);
            assert_eq!(right_ilr.len(), parts - 1);
            let mut square_sum = 0.0_f64;
            for (a, b) in left_ilr.iter().zip(&right_ilr) {
                let difference = (a - b).abs();
                assert!(
                    difference <= bound,
                    "{parts} parts at {offset} ulp: ILR coordinate difference {difference} exceeds {bound}"
                );
                square_sum += difference * difference;
            }
            let ilr_distance = square_sum.sqrt();
            assert!(
                ilr_distance <= bound * (parts_f - 1.0).sqrt(),
                "{parts} parts at {offset} ulp: ILR distance {ilr_distance} exceeds sqrt(D-1)*{bound}"
            );
        }
    }

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
        assert!(aitchison_distance(&[0.5, 0.5], &[0.5, 0.5]).expect("self") < 1e-15);
        assert_eq!(
            aitchison_distance(&[0.5, 0.5], &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]),
            Err(TopicMeasurementError::InvalidComposition)
        );
        assert_eq!(
            aitchison_distance(&[0.0, 1.0], &[0.5, 0.5]),
            Err(TopicMeasurementError::InvalidComposition)
        );
        assert_eq!(
            aitchison_distance(&[0.5, 0.5], &[0.0, 1.0]),
            Err(TopicMeasurementError::InvalidComposition)
        );
    }

    #[test]
    fn single_clr_spike_distance_keeps_the_residual_square_sum_exact() {
        // ALR coordinates [a, 0, ..., 0] give clr a(1 - 1/D) on the first part and
        // -a/D elsewhere, so the distance from the equal-share point is a·sqrt((D-1)/D).
        // One unit square followed by many tiny squares makes a plain residual sum
        // round on every step.
        let parts = 16_384_usize;
        #[allow(clippy::cast_precision_loss)]
        let parts_f = parts as f64;
        let spike = parts_f / (parts_f - 1.0);
        let mut coordinates = vec![0.0_f64; parts - 1];
        coordinates[0] = spike;
        let right = from_additive_log_ratio(&coordinates).expect("closed spike");
        let left = vec![1.0_f64 / parts_f; parts];
        let distance = aitchison_distance(&left, &right).expect("aitchison");
        let expected = spike * ((parts_f - 1.0) / parts_f).sqrt();
        let bound = 8.0 * f64::EPSILON * expected;
        assert!(
            (distance - expected).abs() <= bound,
            "clr distance {distance} differs from {expected} by more than {bound}"
        );
    }
}

#[cfg(test)]
mod normalization_regression {
    use super::{additive_log_ratio, from_additive_log_ratio, isometric_log_ratio};

    /// Preserve positive mass when the dominant weight precedes many small weights.
    #[test]
    fn dominant_first_remains_an_admitted_composition() {
        check_inverse_composition(32_768, 0);
    }

    /// Preserve the equivalent composition with the dominant weight last.
    #[test]
    fn dominant_last_remains_an_admitted_composition() {
        check_inverse_composition(32_768, 32_767);
    }

    /// Preserve the ordinary bounded-dimensional control.
    #[test]
    fn small_control_remains_an_admitted_composition() {
        check_inverse_composition(2, 0);
    }

    /// Equal shares are the ILR origin, so every sequential coordinate is exactly zero.
    #[test]
    fn equal_share_origin_coordinates_are_exact() {
        let dimension = 512_usize;
        #[allow(clippy::cast_precision_loss)]
        let share = 1.0_f64 / (dimension as f64);
        let proportions = vec![share; dimension];
        let coordinates = isometric_log_ratio(&proportions).expect("equal shares");
        assert_eq!(coordinates.len(), dimension - 1);
        let bound = 8.0 * f64::EPSILON * share.ln().abs();
        for coordinate in &coordinates {
            assert!(
                coordinate.abs() <= bound,
                "equal-share ILR coordinates must stay within summation rounding of the exact origin"
            );
        }
    }

    /// Check shape, positivity, finiteness and the existing public simplex contract.
    fn check_inverse_composition(dimension: usize, dominant: usize) {
        let mut coordinates = vec![0.0_f64; dimension];
        coordinates[dominant] = 37.0;
        let proportions = from_additive_log_ratio(&coordinates).expect("finite inverse");
        assert_eq!(proportions.len(), dimension + 1);
        for value in &proportions {
            assert!(value.is_finite());
            assert!(*value > 0.0);
        }
        assert!(additive_log_ratio(&proportions).is_ok());
    }
}
