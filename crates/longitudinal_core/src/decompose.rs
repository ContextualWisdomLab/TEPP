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

fn stable_unit_mean(rows: &[OccasionObservation]) -> Result<f64, LongitudinalError> {
    let scale = rows
        .iter()
        .map(|row| row.score().abs())
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }

    // Normalization keeps every addend in [-1, 1], while Neumaier
    // compensation preserves cancellation that a raw `sum / n` would lose.
    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for row in rows {
        let value = row.score() / scale;
        let next = sum + value;
        if sum.abs() >= value.abs() {
            correction += (sum - next) + value;
        } else {
            correction += (value - next) + sum;
        }
        sum = next;
    }
    let normalized_mean = (sum + correction) / rows.len() as f64;
    let mean = scale * normalized_mean;
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
/// order so recovery tests can pair known truth without extra matching. Unit
/// means are accumulated after max-magnitude normalization so a representable
/// mean is not rejected merely because its raw partial sum exceeds binary64.
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
