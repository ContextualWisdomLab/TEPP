//! Deterministic known-topic data-generating process for recovery studies.

use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::TAU;

use uuid::Uuid;

use crate::configuration::TopicDgpConfig;
use crate::document_process::{DocumentMethodEffect, SimulatedDocument};
use crate::latent_event::LatentEvent;
use crate::relation_process::TrueRelation;
use crate::{SeededRng, SimulationConfig, SimulationError};

/// Seed-domain separator for topic/content/prevalence truth generation.
pub const TOPIC_DGP_SEED_DOMAIN: u64 = 0x544f_5049_435f_4447;

const PREVALENCE_CORRELATION: f64 = 0.20;
const PROBABILITY_TOLERANCE: f64 = 1.0e-10;

/// Known latent topic state and generated term counts for one document.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentTopicTruth {
    document_id: Uuid,
    logistic_normal_coordinates: Vec<f64>,
    topic_mixture: Vec<f64>,
    membership_contribution: Vec<f64>,
    relation_contribution: Vec<f64>,
    method_contribution: Vec<f64>,
    term_counts: Vec<u32>,
}

impl DocumentTopicTruth {
    /// Document identity shared with [`SimulatedDocument`].
    #[must_use]
    pub const fn document_id(&self) -> Uuid {
        self.document_id
    }

    /// True additive-log-ratio logistic-normal coordinates against the final topic.
    #[must_use]
    pub fn logistic_normal_coordinates(&self) -> &[f64] {
        &self.logistic_normal_coordinates
    }

    /// True simplex topic mixture after the logistic-normal transform.
    #[must_use]
    pub fn topic_mixture(&self) -> &[f64] {
        &self.topic_mixture
    }

    /// Weighted multiple-membership contribution to each latent coordinate.
    #[must_use]
    pub fn membership_contribution(&self) -> &[f64] {
        &self.membership_contribution
    }

    /// Incoming true-transition contribution to each latent coordinate.
    #[must_use]
    pub fn relation_contribution(&self) -> &[f64] {
        &self.relation_contribution
    }

    /// Known document-method contribution to each latent coordinate.
    #[must_use]
    pub fn method_contribution(&self) -> &[f64] {
        &self.method_contribution
    }

    /// Generated vocabulary counts in canonical term order.
    #[must_use]
    pub fn term_counts(&self) -> &[u32] {
        &self.term_counts
    }
}

/// Digest-bound topic/content/prevalence truth owned by the simulation boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct TopicTruthManifest {
    seed_domain: u64,
    true_k: u32,
    vocabulary_size: u32,
    document_length: u32,
    topic_term_probabilities: Vec<Vec<f64>>,
    prevalence_intercepts: Vec<f64>,
    prevalence_time_slopes: Vec<f64>,
    prevalence_covariance: Vec<Vec<f64>>,
    document_states: Vec<DocumentTopicTruth>,
}

impl TopicTruthManifest {
    /// Explicit deterministic seed domain used for topic-truth generation.
    #[must_use]
    pub const fn seed_domain(&self) -> u64 {
        self.seed_domain
    }

    /// True number of topics.
    #[must_use]
    pub const fn true_k(&self) -> u32 {
        self.true_k
    }

    /// Vocabulary size shared by every topic/content row.
    #[must_use]
    pub const fn vocabulary_size(&self) -> u32 {
        self.vocabulary_size
    }

    /// Generated token count for every document.
    #[must_use]
    pub const fn document_length(&self) -> u32 {
        self.document_length
    }

    /// True topic-by-term probability matrix.
    #[must_use]
    pub fn topic_term_probabilities(&self) -> &[Vec<f64>] {
        &self.topic_term_probabilities
    }

    /// Baseline prevalence intercepts for the `K - 1` ALR coordinates.
    #[must_use]
    pub fn prevalence_intercepts(&self) -> &[f64] {
        &self.prevalence_intercepts
    }

    /// Event-time prevalence slopes for the `K - 1` ALR coordinates.
    #[must_use]
    pub fn prevalence_time_slopes(&self) -> &[f64] {
        &self.prevalence_time_slopes
    }

    /// Declared logistic-normal covariance matrix for residual document state.
    #[must_use]
    pub fn prevalence_covariance(&self) -> &[Vec<f64>] {
        &self.prevalence_covariance
    }

    /// Per-document latent states and generated term observations.
    #[must_use]
    pub fn document_states(&self) -> &[DocumentTopicTruth] {
        &self.document_states
    }

    /// Verify dimensions, probability geometry, count totals, and document identity.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::ManifestInvariantViolation`] when topic truth
    /// is malformed or no longer aligns one-to-one with manifest documents.
    pub(crate) fn verify_against_documents(
        &self,
        documents: &[SimulatedDocument],
    ) -> Result<(), SimulationError> {
        let k = self.true_k as usize;
        let coordinates = k.saturating_sub(1);
        let vocabulary_size = self.vocabulary_size as usize;
        if k < 2 || vocabulary_size < k || self.document_length == 0 {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        if self.topic_term_probabilities.len() != k
            || self.prevalence_intercepts.len() != coordinates
            || self.prevalence_time_slopes.len() != coordinates
            || self.prevalence_covariance.len() != coordinates
        {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        for row in &self.topic_term_probabilities {
            if row.len() != vocabulary_size
                || !row.iter().all(|value| value.is_finite() && *value > 0.0)
                || (row.iter().sum::<f64>() - 1.0).abs() > PROBABILITY_TOLERANCE
            {
                return Err(SimulationError::ManifestInvariantViolation);
            }
        }
        for row in &self.prevalence_covariance {
            if row.len() != coordinates || !row.iter().all(|value| value.is_finite()) {
                return Err(SimulationError::ManifestInvariantViolation);
            }
        }
        if !self
            .prevalence_intercepts
            .iter()
            .chain(&self.prevalence_time_slopes)
            .all(|value| value.is_finite())
        {
            return Err(SimulationError::ManifestInvariantViolation);
        }

        let document_ids: BTreeSet<_> = documents
            .iter()
            .map(SimulatedDocument::document_id)
            .collect();
        let truth_ids: BTreeSet<_> = self
            .document_states
            .iter()
            .map(DocumentTopicTruth::document_id)
            .collect();
        if document_ids.len() != documents.len()
            || truth_ids.len() != self.document_states.len()
            || truth_ids != document_ids
        {
            return Err(SimulationError::ManifestInvariantViolation);
        }

        for state in &self.document_states {
            if state.logistic_normal_coordinates.len() != coordinates
                || state.topic_mixture.len() != k
                || state.membership_contribution.len() != coordinates
                || state.relation_contribution.len() != coordinates
                || state.method_contribution.len() != coordinates
                || state.term_counts.len() != vocabulary_size
            {
                return Err(SimulationError::ManifestInvariantViolation);
            }
            if !state
                .logistic_normal_coordinates
                .iter()
                .chain(&state.topic_mixture)
                .chain(&state.membership_contribution)
                .chain(&state.relation_contribution)
                .chain(&state.method_contribution)
                .all(|value| value.is_finite())
                || state.topic_mixture.iter().any(|value| *value <= 0.0)
                || (state.topic_mixture.iter().sum::<f64>() - 1.0).abs()
                    > PROBABILITY_TOLERANCE
                || state.term_counts.iter().map(|count| u64::from(*count)).sum::<u64>()
                    != u64::from(self.document_length)
            {
                return Err(SimulationError::ManifestInvariantViolation);
            }
        }
        Ok(())
    }
}

/// Generate the known-topic truth associated with an already generated temporal corpus.
#[must_use]
pub(crate) fn generate_topic_truth(
    config: SimulationConfig,
    events: &[LatentEvent],
    documents: &[SimulatedDocument],
    true_relations: &[TrueRelation],
) -> TopicTruthManifest {
    let topic_config = config.topic_dgp();
    let k = topic_config.true_topic_count() as usize;
    let coordinate_count = k - 1;
    let mut rng = SeededRng::new(config.seed() ^ TOPIC_DGP_SEED_DOMAIN);
    let topic_term_probabilities = topic_term_probabilities(topic_config, &mut rng);
    let prevalence_intercepts = prevalence_intercepts(coordinate_count);
    let prevalence_time_slopes = prevalence_time_slopes(topic_config, coordinate_count);
    let prevalence_covariance = prevalence_covariance(topic_config, coordinate_count);
    let event_ordinals: BTreeMap<_, _> = events
        .iter()
        .map(|event| (event.event_id(), event.ordinal()))
        .collect();
    let incoming_transition_counts = incoming_transition_counts(true_relations);

    let mut document_states = Vec::with_capacity(documents.len());
    for document in documents {
        let ordinal = event_ordinals
            .get(&document.event_id())
            .copied()
            .unwrap_or_default();
        let time_position = normalized_event_position(ordinal, config.event_count());
        let membership_contribution = membership_contribution(document, topic_config, coordinate_count);
        let relation_contribution = relation_contribution(
            document,
            topic_config,
            coordinate_count,
            incoming_transition_counts
                .get(&document.event_id())
                .copied()
                .unwrap_or_default(),
        );
        let method_contribution = method_contribution(document.method_effect(), coordinate_count);
        let residual = correlated_residual(topic_config, coordinate_count, &mut rng);
        let mut coordinates = Vec::with_capacity(coordinate_count);
        for coordinate in 0..coordinate_count {
            coordinates.push(
                prevalence_intercepts[coordinate]
                    + prevalence_time_slopes[coordinate] * time_position
                    + membership_contribution[coordinate]
                    + relation_contribution[coordinate]
                    + method_contribution[coordinate]
                    + residual[coordinate],
            );
        }
        let topic_mixture = logistic_normal_mixture(&coordinates);
        let term_probabilities = mixture_term_probabilities(
            &topic_mixture,
            &topic_term_probabilities,
            topic_config.vocabulary_size() as usize,
        );
        let term_counts = sample_term_counts(
            &mut rng,
            &term_probabilities,
            topic_config.document_length(),
        );
        document_states.push(DocumentTopicTruth {
            document_id: document.document_id(),
            logistic_normal_coordinates: coordinates,
            topic_mixture,
            membership_contribution,
            relation_contribution,
            method_contribution,
            term_counts,
        });
    }

    TopicTruthManifest {
        seed_domain: TOPIC_DGP_SEED_DOMAIN,
        true_k: topic_config.true_topic_count(),
        vocabulary_size: topic_config.vocabulary_size(),
        document_length: topic_config.document_length(),
        topic_term_probabilities,
        prevalence_intercepts,
        prevalence_time_slopes,
        prevalence_covariance,
        document_states,
    }
}

fn topic_term_probabilities(config: TopicDgpConfig, rng: &mut SeededRng) -> Vec<Vec<f64>> {
    let k = config.true_topic_count() as usize;
    let vocabulary_size = config.vocabulary_size() as usize;
    let separation = f64::from(config.topic_separation_bps()) / 10_000.0;
    let mut matrix = Vec::with_capacity(k);
    for topic in 0..k {
        let mut row = Vec::with_capacity(vocabulary_size);
        for term in 0..vocabulary_size {
            let anchor = if term % k == topic { 1.0 + 4.0 * separation } else { 1.0 };
            let jitter = 0.90 + 0.20 * unit_open(rng);
            row.push(anchor * jitter);
        }
        normalize_positive(&mut row);
        matrix.push(row);
    }
    matrix
}

#[allow(clippy::cast_precision_loss)]
fn prevalence_intercepts(coordinate_count: usize) -> Vec<f64> {
    let midpoint = (coordinate_count.saturating_sub(1) as f64) / 2.0;
    (0..coordinate_count)
        .map(|coordinate| ((coordinate as f64) - midpoint) * 0.20)
        .collect()
}

fn prevalence_time_slopes(config: TopicDgpConfig, coordinate_count: usize) -> Vec<f64> {
    let magnitude = f64::from(config.temporal_drift_bps()) / 10_000.0;
    (0..coordinate_count)
        .map(|coordinate| if coordinate % 2 == 0 { magnitude } else { -magnitude })
        .collect()
}

fn prevalence_covariance(config: TopicDgpConfig, coordinate_count: usize) -> Vec<Vec<f64>> {
    let standard_deviation = f64::from(config.latent_standard_deviation_bps()) / 10_000.0;
    let variance = standard_deviation * standard_deviation;
    (0..coordinate_count)
        .map(|row| {
            (0..coordinate_count)
                .map(|column| if row == column { variance } else { variance * PREVALENCE_CORRELATION })
                .collect()
        })
        .collect()
}

fn incoming_transition_counts(relations: &[TrueRelation]) -> BTreeMap<Uuid, u32> {
    let mut counts = BTreeMap::new();
    for relation in relations {
        if relation.kind().is_transition() {
            let count = counts.entry(relation.target_id()).or_insert(0_u32);
            *count = count.saturating_add(1);
        }
    }
    counts
}

fn membership_contribution(
    document: &SimulatedDocument,
    config: TopicDgpConfig,
    coordinate_count: usize,
) -> Vec<f64> {
    let magnitude = f64::from(config.membership_effect_bps()) / 10_000.0;
    (0..coordinate_count)
        .map(|coordinate| {
            document
                .memberships()
                .iter()
                .map(|membership| {
                    let byte = membership.group_id().as_bytes()[coordinate % 16];
                    let centered = (f64::from(byte) / 255.0 - 0.5) * 2.0;
                    let weight = f64::from(membership.weight_bps()) / 10_000.0;
                    centered * weight * magnitude
                })
                .sum()
        })
        .collect()
}

fn relation_contribution(
    _document: &SimulatedDocument,
    config: TopicDgpConfig,
    coordinate_count: usize,
    incoming_transition_count: u32,
) -> Vec<f64> {
    let magnitude = f64::from(config.relation_effect_bps()) / 10_000.0;
    let count = f64::from(incoming_transition_count);
    (0..coordinate_count)
        .map(|coordinate| if coordinate % 2 == 0 { count * magnitude } else { -count * magnitude })
        .collect()
}

fn method_contribution(method: DocumentMethodEffect, coordinate_count: usize) -> Vec<f64> {
    let base = match method {
        DocumentMethodEffect::Original => 0.0,
        DocumentMethodEffect::Revision => 0.05,
        DocumentMethodEffect::Translation => -0.05,
        DocumentMethodEffect::TemplateCopy => 0.025,
    };
    (0..coordinate_count)
        .map(|coordinate| if coordinate % 2 == 0 { base } else { -base })
        .collect()
}

fn correlated_residual(
    config: TopicDgpConfig,
    coordinate_count: usize,
    rng: &mut SeededRng,
) -> Vec<f64> {
    let standard_deviation = f64::from(config.latent_standard_deviation_bps()) / 10_000.0;
    let shared = standard_normal(rng);
    let shared_scale = PREVALENCE_CORRELATION.sqrt();
    let independent_scale = (1.0 - PREVALENCE_CORRELATION).sqrt();
    (0..coordinate_count)
        .map(|_| {
            standard_deviation
                * (shared_scale * shared + independent_scale * standard_normal(rng))
        })
        .collect()
}

fn logistic_normal_mixture(coordinates: &[f64]) -> Vec<f64> {
    let maximum = coordinates.iter().copied().fold(0.0_f64, f64::max);
    let mut masses: Vec<f64> = coordinates
        .iter()
        .map(|coordinate| (*coordinate - maximum).exp())
        .collect();
    masses.push((-maximum).exp());
    normalize_positive(&mut masses);
    masses
}

fn mixture_term_probabilities(
    mixture: &[f64],
    topic_term_probabilities: &[Vec<f64>],
    vocabulary_size: usize,
) -> Vec<f64> {
    let mut probabilities = vec![0.0; vocabulary_size];
    for (topic_weight, topic) in mixture.iter().zip(topic_term_probabilities) {
        for (probability, topic_probability) in probabilities.iter_mut().zip(topic) {
            *probability += topic_weight * topic_probability;
        }
    }
    normalize_positive(&mut probabilities);
    probabilities
}

fn sample_term_counts(rng: &mut SeededRng, probabilities: &[f64], document_length: u32) -> Vec<u32> {
    let mut counts = vec![0_u32; probabilities.len()];
    for _ in 0..document_length {
        let term = sample_categorical(rng, probabilities);
        counts[term] = counts[term].saturating_add(1);
    }
    counts
}

fn sample_categorical(rng: &mut SeededRng, probabilities: &[f64]) -> usize {
    let draw = unit_open(rng);
    let mut cumulative = 0.0;
    for (index, probability) in probabilities
        .iter()
        .take(probabilities.len().saturating_sub(1))
        .enumerate()
    {
        cumulative += probability;
        if draw < cumulative {
            return index;
        }
    }
    probabilities.len().saturating_sub(1)
}

fn normalize_positive(values: &mut [f64]) {
    let total: f64 = values.iter().sum();
    for value in values {
        *value /= total;
    }
}

fn normalized_event_position(ordinal: u32, event_count: u32) -> f64 {
    if event_count <= 1 {
        0.0
    } else {
        2.0 * f64::from(ordinal) / f64::from(event_count - 1) - 1.0
    }
}

#[allow(clippy::cast_precision_loss)]
fn unit_open(rng: &mut SeededRng) -> f64 {
    const DENOMINATOR: f64 = 9_007_199_254_740_993.0;
    let numerator = ((rng.next_u64() >> 11) + 1) as f64;
    numerator / DENOMINATOR
}

fn standard_normal(rng: &mut SeededRng) -> f64 {
    let radius = (-2.0 * unit_open(rng).ln()).sqrt();
    let angle = TAU * unit_open(rng);
    radius * angle.cos()
}

#[cfg(test)]
mod tests {
    use super::{
        PREVALENCE_CORRELATION, TOPIC_DGP_SEED_DOMAIN, logistic_normal_mixture,
        normalized_event_position, sample_categorical, standard_normal, topic_term_probabilities,
    };
    use crate::{SeededRng, TopicDgpConfig};

    #[test]
    fn content_rows_are_positive_normalized_and_seeded() {
        let config = TopicDgpConfig::ci_default();
        let mut first_rng = SeededRng::new(7 ^ TOPIC_DGP_SEED_DOMAIN);
        let mut second_rng = SeededRng::new(7 ^ TOPIC_DGP_SEED_DOMAIN);
        let first = topic_term_probabilities(config, &mut first_rng);
        let second = topic_term_probabilities(config, &mut second_rng);
        assert_eq!(first, second);
        assert!(first.iter().flatten().all(|value| *value > 0.0));
        for row in first {
            assert!((row.iter().sum::<f64>() - 1.0).abs() < 1.0e-12);
        }
        assert!((PREVALENCE_CORRELATION - 0.20).abs() < f64::EPSILON);
    }

    #[test]
    fn transforms_and_sampling_cover_baseline_paths() {
        let mixture = logistic_normal_mixture(&[0.0, 0.0]);
        assert_eq!(mixture.len(), 3);
        assert!((mixture.iter().sum::<f64>() - 1.0).abs() < 1.0e-12);
        assert_eq!(normalized_event_position(0, 1), 0.0);
        assert_eq!(normalized_event_position(0, 3), -1.0);
        assert_eq!(normalized_event_position(2, 3), 1.0);

        let mut categorical_rng = SeededRng::new(1);
        assert_eq!(sample_categorical(&mut categorical_rng, &[0.0, 1.0]), 1);
        let mut normal_a = SeededRng::new(11);
        let mut normal_b = SeededRng::new(11);
        assert_eq!(standard_normal(&mut normal_a), standard_normal(&mut normal_b));
    }
}
