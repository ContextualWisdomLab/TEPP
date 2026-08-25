//! Deterministic joint Gaussian plausible values from an identified precision.

use std::f64::consts::TAU;

use sha2::{Digest, Sha256};
use temporal_core::EventTime;
use uuid::Uuid;

use crate::{JointCoordinatePrecision, TopicMeasurementError, reference::cholesky};

/// Stable algorithm identity for reproducible draw manifests.
pub const JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION: &str =
    "tepp.philox4x32-10-box-muller-cholesky.v1";
const MAX_DRAW_VALUES: usize = 1_000_000;
const PHILOX_MULTIPLIER_0: u32 = 0xD251_1F53;
const PHILOX_MULTIPLIER_1: u32 = 0xCD9E_8D57;
const PHILOX_WEYL_0: u32 = 0x9E37_79B9;
const PHILOX_WEYL_1: u32 = 0xBB67_AE85;

/// Versioned deterministic joint Gaussian plausible-value draw set.
#[derive(Clone, Debug, PartialEq)]
pub struct JointPosteriorDrawSet {
    draw_set_id: String,
    seed: u64,
    document_ids: Vec<Uuid>,
    topic_ids: Vec<Uuid>,
    event_times: Vec<EventTime>,
    draws: Vec<Vec<f64>>,
}

/// One exact fit-bound document plausible value, before artifact provenance binding.
#[derive(Clone, Debug, PartialEq)]
pub struct JointPosteriorPlausibleValue {
    /// Opaque fitted document identity.
    pub document_id: Uuid,
    /// Counter-based posterior draw index.
    pub draw_index: u64,
    /// Event time admitted to the fitted input.
    pub event_time: EventTime,
    /// Full-rank ALR coordinates in the draw set's stable topic order.
    pub logistic_normal_coordinates: Vec<f64>,
}

impl JointPosteriorDrawSet {
    /// Return the SHA-256 identity binding algorithm, seed, basis, fit, and draws.
    #[must_use]
    pub fn draw_set_id(&self) -> &str {
        &self.draw_set_id
    }

    /// Return the explicit counter-based seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Return documents in each draw's outer coordinate order.
    #[must_use]
    pub fn document_ids(&self) -> &[Uuid] {
        &self.document_ids
    }

    /// Return ALR numerator topics followed by the reference topic.
    #[must_use]
    pub fn topic_ids(&self) -> &[Uuid] {
        &self.topic_ids
    }

    /// Return fitted event times in document order.
    #[must_use]
    pub fn event_times(&self) -> &[EventTime] {
        &self.event_times
    }

    /// Return draw-major flattened document ALR coordinates.
    #[must_use]
    pub fn draws(&self) -> &[Vec<f64>] {
        &self.draws
    }

    /// Materialize the exact fit-bound subset of posterior artifact records.
    ///
    /// Run, snapshot, cutoff, activity, lineage, and membership provenance are
    /// deliberately absent and must be bound by the analysis layer before a
    /// complete `tepp.topic_context_posterior.v1` artifact exists.
    ///
    /// # Panics
    ///
    /// Panics only on a platform whose `usize` draw index cannot fit `u64`;
    /// supported targets and the bounded draw contract make that unreachable.
    #[must_use]
    pub fn plausible_values(&self) -> Vec<JointPosteriorPlausibleValue> {
        let coordinate_count = self.topic_ids.len() - 1;
        self.document_ids
            .iter()
            .zip(&self.event_times)
            .enumerate()
            .flat_map(|(document, (document_id, event_time))| {
                self.draws
                    .iter()
                    .enumerate()
                    .map(move |(draw_index, draw)| JointPosteriorPlausibleValue {
                        document_id: *document_id,
                        draw_index: u64::try_from(draw_index).expect("bounded draw index fits u64"),
                        event_time: *event_time,
                        logistic_normal_coordinates: draw
                            [document * coordinate_count..(document + 1) * coordinate_count]
                            .to_vec(),
                    })
            })
            .collect()
    }
}

impl JointCoordinatePrecision {
    /// Generate deterministic joint Gaussian Laplace plausible values.
    ///
    /// Philox4x32-10 maps `(draw_index, normal_block)` counters and the
    /// explicit seed to uniforms. Box-Muller v1 maps those uniforms to standard
    /// normals, and an upper-triangular Cholesky solve applies covariance
    /// `P^-1` without forming an inverse. Counter assignment is independent of
    /// execution order and therefore suitable for a later CPU/GPU parity path.
    ///
    /// # Errors
    ///
    /// Returns a typed error for zero or oversized draw requests and for any
    /// invalid or non-finite precision solve.
    pub fn draw_joint_gaussian(
        &self,
        seed: u64,
        draw_count: usize,
    ) -> Result<JointPosteriorDrawSet, TopicMeasurementError> {
        let dimension = self.values.len();
        let value_count = draw_count
            .checked_mul(dimension)
            .filter(|count| draw_count > 0 && *count <= MAX_DRAW_VALUES)
            .ok_or(TopicMeasurementError::InvalidModelInput)?;
        if self.coordinate_means.len() != dimension {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        let lower = cholesky(&self.values)?;
        let mut draws = Vec::with_capacity(draw_count);
        for draw_index in 0..draw_count {
            let counter =
                u64::try_from(draw_index).map_err(|_| TopicMeasurementError::InvalidModelInput)?;
            let standard = standard_normals(seed, counter, dimension);
            let mut deviation = vec![0.0; dimension];
            for row in (0..dimension).rev() {
                let upper_product = ((row + 1)..dimension)
                    .map(|column| lower[column][row] * deviation[column])
                    .sum::<f64>();
                deviation[row] = (standard[row] - upper_product) / lower[row][row];
            }
            let draw: Vec<f64> = self
                .coordinate_means
                .iter()
                .zip(deviation)
                .map(|(mean, offset)| mean + offset)
                .collect();
            if draw.iter().any(|value| !value.is_finite()) {
                return Err(TopicMeasurementError::NonFiniteEstimate);
            }
            draws.push(draw);
        }
        debug_assert_eq!(draws.iter().map(Vec::len).sum::<usize>(), value_count);
        let draw_set_id = draw_set_digest(self, seed, &draws);
        Ok(JointPosteriorDrawSet {
            draw_set_id,
            seed,
            document_ids: self.document_ids.clone(),
            topic_ids: self.topic_ids.clone(),
            event_times: self.event_times.clone(),
            draws,
        })
    }
}

fn philox4x32_10(mut counter: [u32; 4], mut key: [u32; 2]) -> [u32; 4] {
    for _ in 0..10 {
        let product_0 = u64::from(PHILOX_MULTIPLIER_0) * u64::from(counter[0]);
        let product_1 = u64::from(PHILOX_MULTIPLIER_1) * u64::from(counter[2]);
        let [low_0, high_0] = split_u64(product_0);
        let [low_1, high_1] = split_u64(product_1);
        counter = [
            high_1 ^ counter[1] ^ key[0],
            low_1,
            high_0 ^ counter[3] ^ key[1],
            low_0,
        ];
        key[0] = key[0].wrapping_add(PHILOX_WEYL_0);
        key[1] = key[1].wrapping_add(PHILOX_WEYL_1);
    }
    counter
}

fn split_u64(value: u64) -> [u32; 2] {
    let bytes = value.to_le_bytes();
    [
        u32::from_le_bytes(bytes[..4].try_into().expect("four low bytes")),
        u32::from_le_bytes(bytes[4..].try_into().expect("four high bytes")),
    ]
}

fn standard_normals(seed: u64, draw_index: u64, dimension: usize) -> Vec<f64> {
    let key = split_u64(seed);
    let mut values = Vec::with_capacity(dimension);
    let mut block = 0_u64;
    while values.len() < dimension {
        let [block_low, block_high] = split_u64(block);
        let [draw_low, draw_high] = split_u64(draw_index);
        let words = philox4x32_10([block_low, block_high, draw_low, draw_high], key);
        for pair in [[words[0], words[1]], [words[2], words[3]]] {
            if values.len() == dimension {
                break;
            }
            let first = (f64::from(pair[0]) + 0.5) / 4_294_967_296.0;
            let second = (f64::from(pair[1]) + 0.5) / 4_294_967_296.0;
            let radius = (-2.0 * first.ln()).sqrt();
            let angle = TAU * second;
            values.push(radius * angle.cos());
            if values.len() < dimension {
                values.push(radius * angle.sin());
            }
        }
        block = block.wrapping_add(1);
    }
    values
}

fn draw_set_digest(precision: &JointCoordinatePrecision, seed: u64, draws: &[Vec<f64>]) -> String {
    let mut digest = Sha256::new();
    digest.update(JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION.as_bytes());
    digest.update(seed.to_le_bytes());
    digest.update(
        u64::try_from(draws.len())
            .expect("bounded draw count fits u64")
            .to_le_bytes(),
    );
    for identity in precision.document_ids.iter().chain(&precision.topic_ids) {
        digest.update(identity.as_bytes());
    }
    for event_time in &precision.event_times {
        digest.update(event_time.to_rfc3339().as_bytes());
    }
    for value in precision
        .coordinate_means
        .iter()
        .chain(precision.values.iter().flatten())
        .chain(draws.iter().flatten())
    {
        digest.update(value.to_bits().to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}

#[cfg(test)]
mod tests {
    use super::{JointCoordinatePrecision, philox4x32_10, standard_normals};
    use crate::TopicMeasurementError;
    use temporal_core::EventTime;
    use uuid::Uuid;

    fn precision() -> JointCoordinatePrecision {
        JointCoordinatePrecision {
            document_ids: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
            topic_ids: vec![Uuid::from_u128(11), Uuid::from_u128(12)],
            event_times: vec![
                EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("time"),
                EventTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("time"),
            ],
            coordinate_means: vec![0.25, -0.5],
            values: vec![vec![4.0, 0.0], vec![0.0, 9.0]],
        }
    }

    #[test]
    fn philox_reference_vector_and_draw_identity_are_stable() {
        assert_eq!(
            philox4x32_10([0; 4], [0; 2]),
            [0x6627_e8d5, 0xe169_c58d, 0xbc57_ac4c, 0x9b00_dbd8]
        );
        let first = precision().draw_joint_gaussian(7, 3).expect("draws");
        let repeat = precision().draw_joint_gaussian(7, 3).expect("repeat");
        let other_seed = precision().draw_joint_gaussian(8, 3).expect("other seed");
        assert_eq!(first, repeat);
        assert_ne!(first.draws(), other_seed.draws());
        assert_ne!(first.draw_set_id(), other_seed.draw_set_id());
        assert_eq!(first.draw_set_id().len(), 64);
        assert_eq!(first.seed(), 7);
        assert_eq!(first.document_ids(), precision().document_ids);
        assert_eq!(first.topic_ids(), precision().topic_ids);
        assert_eq!(first.event_times(), precision().event_times);
        let values = first.plausible_values();
        assert_eq!(values.len(), 6);
        assert_eq!(values[0].document_id, Uuid::from_u128(1));
        assert_eq!(values[0].draw_index, 0);
        assert_eq!(values[0].event_time, precision().event_times[0]);
        assert_eq!(values[0].logistic_normal_coordinates.len(), 1);
        for dimension in 1..=6 {
            assert_eq!(standard_normals(7, 0, dimension).len(), dimension);
        }
    }

    #[test]
    fn empirical_covariance_matches_inverse_precision_with_derived_bounds() {
        const DRAW_COUNT: usize = 100_000;
        const DRAW_COUNT_F64: f64 = 100_000.0;
        const FAMILY_ERROR_PROBABILITY: f64 = 0.001;
        const MOMENT_COUNT: f64 = 3.0;
        let target = precision();
        let draws = target.draw_joint_gaussian(19, DRAW_COUNT).expect("draws");
        let centered: Vec<[f64; 2]> = draws
            .draws()
            .iter()
            .map(|draw| [draw[0] - 0.25, draw[1] + 0.5])
            .collect();
        let variance_0 = centered.iter().map(|row| row[0] * row[0]).sum::<f64>() / DRAW_COUNT_F64;
        let variance_1 = centered.iter().map(|row| row[1] * row[1]).sum::<f64>() / DRAW_COUNT_F64;
        let covariance = centered.iter().map(|row| row[0] * row[1]).sum::<f64>() / DRAW_COUNT_F64;
        let expected_0 = 0.25_f64;
        let expected_1 = 1.0 / 9.0;
        let per_moment_error = FAMILY_ERROR_PROBABILITY / MOMENT_COUNT;
        let variance_bound =
            |expected: f64| expected * (2.0 / (DRAW_COUNT_F64 * per_moment_error)).sqrt();
        let covariance_bound =
            (expected_0 * expected_1 / (DRAW_COUNT_F64 * per_moment_error)).sqrt();
        assert!((variance_0 - expected_0).abs() <= variance_bound(expected_0));
        assert!((variance_1 - expected_1).abs() <= variance_bound(expected_1));
        assert!(covariance.abs() <= covariance_bound);
    }

    #[test]
    fn invalid_draw_requests_and_precision_fail_closed() {
        assert_eq!(
            precision().draw_joint_gaussian(1, 0),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        assert_eq!(
            precision().draw_joint_gaussian(1, 500_001),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        let mut invalid = precision();
        invalid.coordinate_means.pop();
        assert_eq!(
            invalid.draw_joint_gaussian(1, 1),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        let mut non_spd = precision();
        non_spd.values[0][0] = 0.0;
        assert_eq!(
            non_spd.draw_joint_gaussian(1, 1),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        let mut non_finite_mean = precision();
        non_finite_mean.coordinate_means[0] = f64::NAN;
        assert_eq!(
            non_finite_mean.draw_joint_gaussian(1, 1),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
    }
}
