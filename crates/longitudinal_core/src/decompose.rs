//! Unit-mean centering that separates between from within residuals.

use std::collections::HashSet;

use crate::{ComponentLevel, ComponentValue, LongitudinalError};

/// One occasion score for one unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OccasionObservation {
    unit_index: u32,
    occasion_index: u32,
    score: f64,
}

impl OccasionObservation {
    /// Construct an occasion score record.
    #[must_use]
    pub const fn new(unit_index: u32, occasion_index: u32, score: f64) -> Self {
        Self {
            unit_index,
            occasion_index,
            score,
        }
    }

    /// Return the unit index.
    #[must_use]
    pub const fn unit_index(self) -> u32 {
        self.unit_index
    }

    /// Return the occasion index.
    #[must_use]
    pub const fn occasion_index(self) -> u32 {
        self.occasion_index
    }

    /// Return the observed score.
    #[must_use]
    pub const fn score(self) -> f64 {
        self.score
    }
}

fn same_sign_unit_mean(values: &[f64]) -> Result<f64, LongitudinalError> {
    let mut mean = 0.0_f64;
    for (index, &value) in values.iter().enumerate() {
        mean += (value - mean) / (index + 1) as f64;
    }
    if mean.is_finite() {
        Ok(mean)
    } else {
        Err(LongitudinalError::InvalidObservationPayload)
    }
}

fn stable_unit_mean(rows: &[OccasionObservation]) -> Result<f64, LongitudinalError> {
    let values: Vec<f64> = rows.iter().map(|row| row.score()).collect();
    let mut positives: Vec<f64> = values.iter().copied().filter(|value| *value > 0.0).collect();
    let mut negatives: Vec<f64> = values.iter().copied().filter(|value| *value < 0.0).collect();

    if positives.is_empty() && negatives.is_empty() {
        return Ok(0.0);
    }
    if positives.is_empty() || negatives.is_empty() {
        return same_sign_unit_mean(&values);
    }

    positives.sort_by(|left, right| right.total_cmp(left));
    negatives.sort_by(|left, right| left.total_cmp(right));

    let mut positive_index = 0_usize;
    let mut negative_index = 0_usize;
    let mut positive = positives[0];
    let mut negative = negatives[0];
    let mut residuals = Vec::with_capacity(values.len());

    loop {
        let residual = positive + negative;
        if residual > 0.0 {
            positive = residual;
            negative_index += 1;
            if negative_index == negatives.len() {
                residuals.push(positive);
                residuals.extend_from_slice(&positives[positive_index + 1..]);
                break;
            }
            negative = negatives[negative_index];
        } else if residual < 0.0 {
            negative = residual;
            positive_index += 1;
            if positive_index == positives.len() {
                residuals.push(negative);
                residuals.extend_from_slice(&negatives[negative_index + 1..]);
                break;
            }
            positive = positives[positive_index];
        } else {
            positive_index += 1;
            negative_index += 1;
            if positive_index == positives.len() || negative_index == negatives.len() {
                residuals.extend_from_slice(&positives[positive_index..]);
                residuals.extend_from_slice(&negatives[negative_index..]);
                break;
            }
            positive = positives[positive_index];
            negative = negatives[negative_index];
        }
    }

    if residuals.is_empty() {
        return Ok(0.0);
    }
    let residual_mean = same_sign_unit_mean(&residuals)?;
    let retained_count = residuals.len() as f64;
    let total_count = values.len() as f64;
    let retained_mass = residual_mean * retained_count;
    let mean = if retained_mass.is_finite() {
        retained_mass / total_count
    } else {
        (residual_mean / total_count) * retained_count
    };
    if mean.is_finite() {
        Ok(mean)
    } else {
        Err(LongitudinalError::InvalidObservationPayload)
    }
}

/// Decompose occasion scores into unit means and within residuals.
///
/// Each unit contributes one between component at occasion `0` and one within
/// residual per observed occasion. Units and occasions are emitted in sorted
/// order so recovery tests can pair known truth without extra matching. Mixed-
/// sign unit means cancel opposite extreme values before averaging retained
/// mass, preserving representable low-order and subnormal evidence without raw
/// sum overflow or max-scale normalization underflow.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidObservationPayload`] when fewer than two
/// units are present, any unit has fewer than two occasions, a `(unit,
/// occasion)` pair is duplicated, a score is non-finite, or a resulting mean
/// or within residual is not representable.
pub fn decompose_within_between(
    observations: &[OccasionObservation],
) -> Result<Vec<ComponentValue>, LongitudinalError> {
    if observations.len() < 4 {
        return Err(LongitudinalError::InvalidObservationPayload);
    }
    let mut rows: Vec<OccasionObservation> = Vec::with_capacity(observations.len());
    let mut seen_pairs: HashSet<(u32, u32)> = HashSet::with_capacity(observations.len());
    for observation in observations {
        if !observation.score().is_finite() {
            return Err(LongitudinalError::InvalidObservationPayload);
        }
        if !seen_pairs.insert((observation.unit_index(), observation.occasion_index())) {
            return Err(LongitudinalError::InvalidObservationPayload);
        }
        rows.push(*observation);
    }
    rows.sort_by_key(|row| (row.unit_index(), row.occasion_index()));

    let mut unit_starts: Vec<(u32, usize, usize)> = Vec::new();
    let mut cursor = 0_usize;
    while cursor < rows.len() {
        let unit = rows[cursor].unit_index();
        let start = cursor;
        cursor += 1;
        while cursor < rows.len() && rows[cursor].unit_index() == unit {
            cursor += 1;
        }
        unit_starts.push((unit, start, cursor));
    }
    if unit_starts.len() < 2 {
        return Err(LongitudinalError::InvalidObservationPayload);
    }

    let mut components = Vec::new();
    for &(unit, start, end) in &unit_starts {
        let count = end - start;
        if count < 2 {
            return Err(LongitudinalError::InvalidObservationPayload);
        }
        let mean = stable_unit_mean(&rows[start..end])?;
        components.push(ComponentValue::new(unit, 0, ComponentLevel::Between, mean));
        for row in &rows[start..end] {
            let residual = row.score() - mean;
            if !residual.is_finite() {
                return Err(LongitudinalError::InvalidObservationPayload);
            }
            components.push(ComponentValue::new(
                unit,
                row.occasion_index(),
                ComponentLevel::Within,
                residual,
            ));
        }
    }
    Ok(components)
}

#[cfg(test)]
mod tests {
    use super::{OccasionObservation, decompose_within_between};
    use crate::{ComponentLevel, LongitudinalError};

    #[test]
    fn sparse_duplicate_and_nan_fail_closed() {
        let one_unit = [
            OccasionObservation::new(0, 0, 1.0),
            OccasionObservation::new(0, 1, 2.0),
        ];
        assert_eq!(
            decompose_within_between(&one_unit),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        let one_unit_long = [
            OccasionObservation::new(0, 0, 1.0),
            OccasionObservation::new(0, 1, 2.0),
            OccasionObservation::new(0, 2, 3.0),
            OccasionObservation::new(0, 3, 4.0),
        ];
        assert_eq!(
            decompose_within_between(&one_unit_long),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        let short_unit = [
            OccasionObservation::new(0, 0, 1.0),
            OccasionObservation::new(0, 1, 2.0),
            OccasionObservation::new(1, 0, 3.0),
            OccasionObservation::new(1, 1, 4.0),
            OccasionObservation::new(2, 0, 5.0),
        ];
        assert_eq!(
            decompose_within_between(&short_unit),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        let duplicate = [
            OccasionObservation::new(0, 0, 1.0),
            OccasionObservation::new(0, 0, 2.0),
            OccasionObservation::new(1, 0, 3.0),
            OccasionObservation::new(1, 1, 4.0),
        ];
        assert_eq!(
            decompose_within_between(&duplicate),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        let nan = [
            OccasionObservation::new(0, 0, f64::NAN),
            OccasionObservation::new(0, 1, 2.0),
            OccasionObservation::new(1, 0, 3.0),
            OccasionObservation::new(1, 1, 4.0),
        ];
        assert_eq!(
            decompose_within_between(&nan),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        let recovered = decompose_within_between(&[
            OccasionObservation::new(1, 1, 4.0),
            OccasionObservation::new(0, 1, 2.0),
            OccasionObservation::new(1, 0, 2.0),
            OccasionObservation::new(0, 0, 0.0),
        ])
        .expect("sorted");
        assert_eq!(recovered[0].level(), ComponentLevel::Between);
        assert_eq!(recovered[0].unit_index(), 0);
        assert!((recovered[0].value() - 1.0).abs() < f64::EPSILON);
        assert_eq!(OccasionObservation::new(9, 8, 0.0).unit_index(), 9);
    }

    #[test]
    fn representable_unit_mean_survives_raw_sum_overflow() {
        let recovered = decompose_within_between(&[
            OccasionObservation::new(0, 0, f64::MAX),
            OccasionObservation::new(0, 1, f64::MAX),
            OccasionObservation::new(1, 0, 0.0),
            OccasionObservation::new(1, 1, 0.0),
        ])
        .expect("representable unit mean must not fail on an overflowing partial sum");

        assert_eq!(recovered[0].level(), ComponentLevel::Between);
        assert_eq!(recovered[0].value(), f64::MAX);
        assert_eq!(recovered[1].value(), 0.0);
        assert_eq!(recovered[2].value(), 0.0);
    }

    #[test]
    fn representable_subnormal_unit_mean_survives_extreme_cancellation() {
        let minimum_subnormal = f64::from_bits(1);
        let recovered = decompose_within_between(&[
            OccasionObservation::new(0, 0, f64::MAX),
            OccasionObservation::new(0, 1, -f64::MAX),
            OccasionObservation::new(0, 2, f64::from_bits(4)),
            OccasionObservation::new(1, 0, 0.0),
            OccasionObservation::new(1, 1, 0.0),
        ])
        .expect("a representable subnormal unit mean must survive extreme cancellation");

        assert_eq!(recovered[0].level(), ComponentLevel::Between);
        assert_eq!(recovered[0].value().to_bits(), minimum_subnormal.to_bits());
    }
}
