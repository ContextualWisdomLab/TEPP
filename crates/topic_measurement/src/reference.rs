//! Bounded CPU `f64` reference estimator for TRSL-TM prevalence and lineage.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use corpus_split::CorpusSnapshot;
use membership_core::{GroupId, MemberId, MembershipNetwork, MembershipRole};
use relation_graph::RelationGraph;
use temporal_core::EventTime;
use uuid::Uuid;

use crate::{SparseMatrix, TopicMeasurementError, from_additive_log_ratio};

const DEFAULT_PRIOR_VARIANCE: f64 = 1.0;
const DEFAULT_RELATION_STRENGTH: f64 = 0.25;
const DEFAULT_RIDGE: f64 = 0.01;
const DEFAULT_TOPIC_SMOOTHING: f64 = 0.05;
const DEFAULT_STEP_SIZE: f64 = 0.2;

/// One column in the structural prevalence mean.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrevalenceFeature {
    /// Constant intercept.
    Intercept,
    /// Standardized event-time offset.
    EventTime,
    /// Caller-supplied admitted prevalence covariate.
    Covariate(usize),
    /// One active weighted cross-classified membership context.
    Membership {
        /// Contextual membership role.
        role: MembershipRole,
        /// Opaque analytical group identity.
        group_id: GroupId,
    },
}

/// Validated input for the CPU `f64` reference estimator.
#[derive(Clone, Debug)]
pub struct ReferenceTopicInput {
    document_ids: Vec<Uuid>,
    term_rows: Vec<Vec<(usize, f64)>>,
    vocabulary_size: usize,
    design: Vec<Vec<f64>>,
    features: Vec<PrevalenceFeature>,
    transition_pairs: Vec<(usize, usize)>,
}

impl ReferenceTopicInput {
    /// Build a cutoff-, membership-, time-, and relation-validated model input.
    ///
    /// `document_term` may be CSR or CSC. `covariates`, when present, may also
    /// use either orientation. Every document must occur in `snapshot`, have a
    /// nonempty nonnegative term row, span at least two event times, and have at
    /// least one active membership across the modeled corpus. Only validated
    /// forward transition edges with both endpoints in the corpus affect the
    /// relational objective; all other relation kinds remain provenance only.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] when any dimension,
    /// cutoff, count, time, membership, covariate, or transition invariant fails.
    pub fn new(
        snapshot: &CorpusSnapshot,
        document_ids: Vec<Uuid>,
        document_term: &SparseMatrix,
        event_times: &[EventTime],
        covariates: Option<&SparseMatrix>,
        memberships: &MembershipNetwork,
        relations: &RelationGraph,
    ) -> Result<Self, TopicMeasurementError> {
        let document_count = document_ids.len();
        if document_count < 2
            || document_term.rows() != document_count
            || document_term.columns() < 2
            || event_times.len() != document_count
            || document_ids.iter().any(|id| !snapshot.contains(*id))
        {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        let index_by_id: HashMap<Uuid, usize> = document_ids
            .iter()
            .copied()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect();
        if index_by_id.len() != document_count {
            return Err(TopicMeasurementError::InvalidModelInput);
        }

        let term_rows = document_term.row_entries();
        for row in &term_rows {
            if row.is_empty()
                || row.iter().any(|(_, value)| *value < 0.0)
                || row.iter().map(|(_, value)| value).sum::<f64>() <= 0.0
            {
                return Err(TopicMeasurementError::InvalidModelInput);
            }
        }

        let (design, features) = build_design(&document_ids, event_times, covariates, memberships)?;
        let transition_pairs = collect_transition_pairs(&index_by_id, relations)?;

        Ok(Self {
            document_ids,
            term_rows,
            vocabulary_size: document_term.columns(),
            design,
            features,
            transition_pairs,
        })
    }

    /// Return the number of modeled documents.
    #[must_use]
    pub fn document_count(&self) -> usize {
        self.document_ids.len()
    }

    /// Return the vocabulary size.
    #[must_use]
    pub const fn vocabulary_size(&self) -> usize {
        self.vocabulary_size
    }

    /// Return the ordered structural prevalence features.
    #[must_use]
    pub fn features(&self) -> &[PrevalenceFeature] {
        &self.features
    }
}

fn build_design(
    document_ids: &[Uuid],
    event_times: &[EventTime],
    covariates: Option<&SparseMatrix>,
    memberships: &MembershipNetwork,
) -> Result<(Vec<Vec<f64>>, Vec<PrevalenceFeature>), TopicMeasurementError> {
    let document_count = document_ids.len();
    let standardized_time = standardize_event_time(event_times)?;
    let covariate_rows = match covariates {
        Some(matrix) if matrix.rows() == document_count => Some(matrix.row_entries()),
        Some(_) => return Err(TopicMeasurementError::InvalidModelInput),
        None => None,
    };
    let covariate_count = covariate_rows.as_ref().map_or(0, |rows| {
        rows.iter()
            .flatten()
            .map(|(column, _)| *column)
            .max()
            .map_or(0, |value| value + 1)
    });

    let active: Vec<_> = document_ids
        .iter()
        .zip(event_times)
        .map(|(id, time)| memberships.active_memberships_for(MemberId::from_uuid(*id), *time))
        .collect();
    let membership_keys: BTreeSet<_> = active
        .iter()
        .flatten()
        .map(|assignment| (assignment.role(), assignment.group_id()))
        .collect();
    if membership_keys.is_empty() {
        return Err(TopicMeasurementError::InvalidModelInput);
    }
    let membership_columns: BTreeMap<_, _> = membership_keys
        .iter()
        .copied()
        .enumerate()
        .map(|(index, key)| (key, index))
        .collect();

    let mut features = vec![PrevalenceFeature::Intercept, PrevalenceFeature::EventTime];
    features.extend((0..covariate_count).map(PrevalenceFeature::Covariate));
    features.extend(
        membership_keys
            .iter()
            .map(|(role, group_id)| PrevalenceFeature::Membership {
                role: *role,
                group_id: *group_id,
            }),
    );
    let mut design = vec![vec![0.0; features.len()]; document_count];
    for row in 0..document_count {
        design[row][0] = 1.0;
        design[row][1] = standardized_time[row];
        if let Some(covariates) = &covariate_rows {
            for &(column, value) in &covariates[row] {
                design[row][2 + column] = value;
            }
        }
        for assignment in &active[row] {
            let column = membership_columns[&(assignment.role(), assignment.group_id())];
            design[row][2 + covariate_count + column] = assignment.weight().value();
        }
    }
    Ok((design, features))
}

fn collect_transition_pairs(
    index_by_id: &HashMap<Uuid, usize>,
    relations: &RelationGraph,
) -> Result<Vec<(usize, usize)>, TopicMeasurementError> {
    let mut transition_pairs = BTreeSet::new();
    for edge in relations.edges().filter(|edge| edge.is_transition_edge()) {
        let Some(&source) = index_by_id.get(&edge.source().as_uuid()) else {
            continue;
        };
        let Some(&target) = index_by_id.get(&edge.target().as_uuid()) else {
            continue;
        };
        transition_pairs.insert((source, target));
    }
    if transition_pairs.is_empty() {
        Err(TopicMeasurementError::InvalidModelInput)
    } else {
        Ok(transition_pairs.into_iter().collect())
    }
}

/// Bounded deterministic reference-estimator configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceTopicModelConfig {
    topic_count: usize,
    seeds: Vec<u64>,
    maximum_iterations: usize,
    tolerance: f64,
    prior_variance: f64,
    relation_strength: f64,
    ridge: f64,
    topic_smoothing: f64,
    step_size: f64,
}

impl ReferenceTopicModelConfig {
    /// Construct a reference configuration with ADR-owned v1 hyperparameters.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] unless `topic_count`
    /// is at least two, seeds are nonempty, the iteration budget is at least
    /// two, and tolerance is finite and positive.
    pub fn new(
        topic_count: usize,
        seeds: Vec<u64>,
        maximum_iterations: usize,
        tolerance: f64,
    ) -> Result<Self, TopicMeasurementError> {
        let value = Self {
            topic_count,
            seeds,
            maximum_iterations,
            tolerance,
            prior_variance: DEFAULT_PRIOR_VARIANCE,
            relation_strength: DEFAULT_RELATION_STRENGTH,
            ridge: DEFAULT_RIDGE,
            topic_smoothing: DEFAULT_TOPIC_SMOOTHING,
            step_size: DEFAULT_STEP_SIZE,
        };
        value.validate()?;
        Ok(value)
    }

    /// Replace numerical hyperparameters while retaining dimensional controls.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] for any non-finite
    /// or non-positive value, except `relation_strength` and `ridge`, which may
    /// be exactly zero for a declared ablation.
    pub fn with_hyperparameters(
        mut self,
        prior_variance: f64,
        relation_strength: f64,
        ridge: f64,
        topic_smoothing: f64,
        step_size: f64,
    ) -> Result<Self, TopicMeasurementError> {
        self.prior_variance = prior_variance;
        self.relation_strength = relation_strength;
        self.ridge = ridge;
        self.topic_smoothing = topic_smoothing;
        self.step_size = step_size;
        self.validate()?;
        Ok(self)
    }

    fn validate(&self) -> Result<(), TopicMeasurementError> {
        if self.topic_count < 2
            || self.seeds.is_empty()
            || self.maximum_iterations < 2
            || !self.tolerance.is_finite()
            || self.tolerance <= 0.0
            || !self.prior_variance.is_finite()
            || self.prior_variance <= 0.0
            || !self.relation_strength.is_finite()
            || self.relation_strength < 0.0
            || !self.ridge.is_finite()
            || self.ridge < 0.0
            || !self.topic_smoothing.is_finite()
            || self.topic_smoothing <= 0.0
            || !self.step_size.is_finite()
            || self.step_size <= 0.0
        {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        Ok(())
    }
}

/// One inferred predecessor/successor association within a dominant topic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopicSequenceEdge {
    /// Opaque predecessor document identity.
    pub predecessor_document_id: Uuid,
    /// Opaque successor document identity.
    pub successor_document_id: Uuid,
    /// Artifact-local global topic index.
    pub topic_index: usize,
    /// Minimum dominant-topic posterior mean across the two documents.
    pub association_strength: f64,
}

/// A converged topic-model result with uncertainty and lineage counts.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceTopicModel {
    /// Selected deterministic initialization seed.
    pub seed: u64,
    /// Iterations used by the selected converged fit.
    pub iterations: usize,
    /// Final finite penalized objective.
    pub objective: f64,
    /// Global topic-by-term probability matrix.
    pub topic_term_probabilities: Vec<Vec<f64>>,
    /// Document-by-topic posterior mean proportions.
    pub document_topic_proportions: Vec<Vec<f64>>,
    /// Diagonal Laplace variance for each document ALR coordinate.
    pub document_coordinate_variances: Vec<Vec<f64>>,
    /// Structural prevalence coefficient matrix, feature by ALR coordinate.
    pub prevalence_coefficients: Vec<Vec<f64>>,
    /// Ordered structural feature meanings for coefficient rows.
    pub prevalence_features: Vec<PrevalenceFeature>,
    /// Inferred dominant-topic links restricted to explicit forward transitions.
    pub sequence_edges: Vec<TopicSequenceEdge>,
    /// Distinct documents incident to at least one inferred sequence edge.
    pub connected_post_count: usize,
    /// Distinct global topics represented by at least one sequence edge.
    pub lineage_count: usize,
}

#[derive(Clone)]
struct FitState {
    seed: u64,
    iterations: usize,
    objective: f64,
    beta: Vec<Vec<f64>>,
    eta: Vec<Vec<f64>>,
    coefficients: Vec<Vec<f64>>,
}

/// Fit the bounded deterministic CPU `f64` TRSL-TM reference estimator.
///
/// # Errors
///
/// Returns a typed invalid-input, non-finite, or convergence failure. The
/// function never returns a partial fit.
pub fn fit_reference_topic_model(
    input: &ReferenceTopicInput,
    config: &ReferenceTopicModelConfig,
) -> Result<ReferenceTopicModel, TopicMeasurementError> {
    config.validate()?;
    if config.topic_count > input.vocabulary_size {
        return Err(TopicMeasurementError::InvalidModelInput);
    }
    let mut best = None;
    for &seed in &config.seeds {
        match fit_seed(input, config, seed) {
            Ok(candidate)
                if best.as_ref().is_none_or(|incumbent: &FitState| {
                    candidate.objective > incumbent.objective
                }) =>
            {
                best = Some(candidate);
            }
            Ok(_) | Err(TopicMeasurementError::DidNotConverge) => {}
            Err(error) => return Err(error),
        }
    }
    let state = best.ok_or(TopicMeasurementError::DidNotConverge)?;
    build_result(input, config, state)
}

fn fit_seed(
    input: &ReferenceTopicInput,
    config: &ReferenceTopicModelConfig,
    seed: u64,
) -> Result<FitState, TopicMeasurementError> {
    let document_count = input.document_ids.len();
    let coordinate_count = config.topic_count - 1;
    let mut rng = seed.max(1);
    let mut beta = vec![vec![0.0; input.vocabulary_size]; config.topic_count];
    for topic in &mut beta {
        for value in topic.iter_mut() {
            *value = config.topic_smoothing + next_unit(&mut rng);
        }
        normalize(topic)?;
    }
    let mut eta = vec![vec![0.0; coordinate_count]; document_count];
    for row in &mut eta {
        for value in row {
            *value = (next_unit(&mut rng) - 0.5) * 0.1;
        }
    }
    let mut coefficients = vec![vec![0.0; coordinate_count]; input.features.len()];
    let mut previous = None;

    for iteration in 1..=config.maximum_iterations {
        let theta = topic_proportions(&eta)?;
        let (document_topic_counts, beta_counts, ll) =
            expectation(input, &theta, &beta, config.topic_count)?;
        let means = prevalence_means(&input.design, &coefficients);
        let objective = objective(input, config, &theta, &eta, &means, &coefficients, ll)?;
        if previous.is_some_and(|value: f64| {
            (objective - value).abs() / (1.0 + value.abs()) <= config.tolerance
        }) && iteration > 3
        {
            return Ok(FitState {
                seed,
                iterations: iteration,
                objective,
                beta,
                eta,
                coefficients,
            });
        }
        previous = Some(objective);
        beta = update_beta(beta_counts, config.topic_smoothing)?;
        update_coefficients(input, config, &eta, &means, &mut coefficients)?;
        let counts = &document_topic_counts;
        update_eta(input, config, counts, &theta, &means, &mut eta)?;
    }
    Err(TopicMeasurementError::DidNotConverge)
}

fn topic_proportions(eta: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, TopicMeasurementError> {
    eta.iter().map(|row| from_additive_log_ratio(row)).collect()
}

type ExpectationOutput = (Vec<Vec<f64>>, Vec<Vec<f64>>, f64);

fn expectation(
    input: &ReferenceTopicInput,
    theta: &[Vec<f64>],
    beta: &[Vec<f64>],
    topic_count: usize,
) -> Result<ExpectationOutput, TopicMeasurementError> {
    let mut document_topic_counts = vec![vec![0.0; topic_count]; input.document_ids.len()];
    let mut beta_counts = vec![vec![0.0; input.vocabulary_size]; topic_count];
    let mut log_likelihood = 0.0;
    for (document, terms) in input.term_rows.iter().enumerate() {
        for &(term, count) in terms {
            let probability = (0..topic_count)
                .map(|topic| theta[document][topic] * beta[topic][term])
                .sum::<f64>();
            let log_probability = probability.ln();
            require_finite(log_probability)?;
            log_likelihood += count * log_probability;
            for topic in 0..topic_count {
                let expected = count * theta[document][topic] * beta[topic][term] / probability;
                document_topic_counts[document][topic] += expected;
                beta_counts[topic][term] += expected;
            }
        }
    }
    if !log_likelihood.is_finite() {
        return Err(TopicMeasurementError::NonFiniteEstimate);
    }
    Ok((document_topic_counts, beta_counts, log_likelihood))
}

fn update_beta(
    mut counts: Vec<Vec<f64>>,
    smoothing: f64,
) -> Result<Vec<Vec<f64>>, TopicMeasurementError> {
    for topic in &mut counts {
        for value in topic.iter_mut() {
            *value += smoothing;
        }
        normalize(topic)?;
    }
    Ok(counts)
}

fn prevalence_means(design: &[Vec<f64>], coefficients: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let coordinate_count = coefficients[0].len();
    design
        .iter()
        .map(|row| {
            let mut mean = vec![0.0; coordinate_count];
            for (feature, value) in row.iter().enumerate() {
                for (coordinate, target) in mean.iter_mut().enumerate() {
                    *target += value * coefficients[feature][coordinate];
                }
            }
            mean
        })
        .collect()
}

fn objective(
    input: &ReferenceTopicInput,
    config: &ReferenceTopicModelConfig,
    theta: &[Vec<f64>],
    eta: &[Vec<f64>],
    means: &[Vec<f64>],
    coefficients: &[Vec<f64>],
    log_likelihood: f64,
) -> Result<f64, TopicMeasurementError> {
    let prior = eta
        .iter()
        .zip(means)
        .flat_map(|(row, mean)| row.iter().zip(mean))
        .map(|(value, mean)| (value - mean).powi(2))
        .sum::<f64>()
        / (2.0 * config.prior_variance);
    let relation = input
        .transition_pairs
        .iter()
        .map(|&(source, target)| {
            theta[source]
                .iter()
                .zip(&theta[target])
                .map(|(left, right)| (left - right).powi(2))
                .sum::<f64>()
        })
        .sum::<f64>()
        * config.relation_strength
        / 2.0;
    let ridge = coefficients
        .iter()
        .flatten()
        .map(|value| value * value)
        .sum::<f64>()
        * config.ridge
        / 2.0;
    let value = log_likelihood - prior - relation - ridge;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(TopicMeasurementError::NonFiniteEstimate)
    }
}

fn update_coefficients(
    input: &ReferenceTopicInput,
    config: &ReferenceTopicModelConfig,
    eta: &[Vec<f64>],
    means: &[Vec<f64>],
    coefficients: &mut [Vec<f64>],
) -> Result<(), TopicMeasurementError> {
    let scale = config.step_size / bounded_count(input.document_ids.len())?;
    for feature in 0..coefficients.len() {
        for coordinate in 0..coefficients[feature].len() {
            let gradient = input
                .design
                .iter()
                .enumerate()
                .map(|(document, row)| {
                    row[feature] * (eta[document][coordinate] - means[document][coordinate])
                        / config.prior_variance
                })
                .sum::<f64>()
                - config.ridge * coefficients[feature][coordinate];
            coefficients[feature][coordinate] += scale * gradient;
            require_finite(coefficients[feature][coordinate])?;
        }
    }
    Ok(())
}

fn update_eta(
    input: &ReferenceTopicInput,
    config: &ReferenceTopicModelConfig,
    counts: &[Vec<f64>],
    theta: &[Vec<f64>],
    means: &[Vec<f64>],
    eta: &mut [Vec<f64>],
) -> Result<(), TopicMeasurementError> {
    let coordinate_count = config.topic_count - 1;
    let mut relation_gradient = vec![vec![0.0; coordinate_count]; input.document_ids.len()];
    for &(source, target) in &input.transition_pairs {
        let delta: Vec<f64> = theta[source]
            .iter()
            .zip(&theta[target])
            .map(|(left, right)| left - right)
            .collect();
        let source_dot = dot(&delta, &theta[source]);
        let target_dot = dot(&delta, &theta[target]);
        for coordinate in 0..coordinate_count {
            relation_gradient[source][coordinate] -= config.relation_strength
                * theta[source][coordinate]
                * (delta[coordinate] - source_dot);
            relation_gradient[target][coordinate] += config.relation_strength
                * theta[target][coordinate]
                * (delta[coordinate] - target_dot);
        }
    }
    for document in 0..eta.len() {
        let token_count = counts[document].iter().sum::<f64>();
        let scale = config.step_size / (1.0 + token_count);
        for coordinate in 0..coordinate_count {
            let gradient = counts[document][coordinate]
                - token_count * theta[document][coordinate]
                - (eta[document][coordinate] - means[document][coordinate]) / config.prior_variance
                + relation_gradient[document][coordinate];
            eta[document][coordinate] += scale * gradient;
            require_finite(eta[document][coordinate])?;
        }
    }
    Ok(())
}

fn build_result(
    input: &ReferenceTopicInput,
    config: &ReferenceTopicModelConfig,
    state: FitState,
) -> Result<ReferenceTopicModel, TopicMeasurementError> {
    let theta = topic_proportions(&state.eta)?;
    let mut degrees = vec![0_usize; input.document_ids.len()];
    for &(source, target) in &input.transition_pairs {
        degrees[source] += 1;
        degrees[target] += 1;
    }
    let mut variances = Vec::with_capacity(theta.len());
    for (document, proportions) in theta.iter().enumerate() {
        let token_count = input.term_rows[document]
            .iter()
            .map(|(_, count)| count)
            .sum::<f64>();
        let degree = bounded_count(degrees[document])?;
        variances.push(
            proportions[..config.topic_count - 1]
                .iter()
                .map(|value| {
                    1.0 / (token_count * value * (1.0 - value)
                        + 1.0 / config.prior_variance
                        + degree * config.relation_strength)
                })
                .collect(),
        );
    }
    let dominant: Vec<usize> = theta.iter().map(|row| argmax(row)).collect();
    let mut sequence_edges = Vec::new();
    let mut connected = BTreeSet::new();
    let mut lineages = BTreeSet::new();
    for &(source, target) in &input.transition_pairs {
        if dominant[source] != dominant[target] {
            continue;
        }
        let topic = dominant[source];
        connected.insert(input.document_ids[source]);
        connected.insert(input.document_ids[target]);
        lineages.insert(topic);
        sequence_edges.push(TopicSequenceEdge {
            predecessor_document_id: input.document_ids[source],
            successor_document_id: input.document_ids[target],
            topic_index: topic,
            association_strength: theta[source][topic].min(theta[target][topic]),
        });
    }
    Ok(ReferenceTopicModel {
        seed: state.seed,
        iterations: state.iterations,
        objective: state.objective,
        topic_term_probabilities: state.beta,
        document_topic_proportions: theta,
        document_coordinate_variances: variances,
        prevalence_coefficients: state.coefficients,
        prevalence_features: input.features.clone(),
        sequence_edges,
        connected_post_count: connected.len(),
        lineage_count: lineages.len(),
    })
}

#[allow(clippy::cast_precision_loss)]
fn standardize_event_time(times: &[EventTime]) -> Result<Vec<f64>, TopicMeasurementError> {
    let origin = times[0].instant().as_nanosecond();
    let offsets: Vec<f64> = times
        .iter()
        .map(|time| (time.instant().as_nanosecond() - origin) as f64 / 1_000_000_000.0)
        .collect();
    let mean = offsets.iter().sum::<f64>() / offsets.len() as f64;
    let variance = offsets
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / offsets.len() as f64;
    if variance <= 0.0 {
        return Err(TopicMeasurementError::InvalidModelInput);
    }
    let deviation = variance.sqrt();
    Ok(offsets
        .iter()
        .map(|value| (value - mean) / deviation)
        .collect())
}

fn normalize(values: &mut [f64]) -> Result<(), TopicMeasurementError> {
    let sum = values.iter().sum::<f64>();
    require_finite(sum.ln())?;
    for value in values {
        *value /= sum;
    }
    Ok(())
}

fn require_finite(value: f64) -> Result<(), TopicMeasurementError> {
    value
        .is_finite()
        .then_some(())
        .ok_or(TopicMeasurementError::NonFiniteEstimate)
}

fn bounded_count(value: usize) -> Result<f64, TopicMeasurementError> {
    u32::try_from(value)
        .map(f64::from)
        .map_err(|_| TopicMeasurementError::InvalidModelInput)
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn argmax(values: &[f64]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map_or(0, |(index, _)| index)
}

fn next_unit(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    #[allow(clippy::cast_precision_loss)]
    let value = (*state >> 11) as f64 / ((1_u64 << 53) as f64);
    value.max(f64::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::{
        FitState, PrevalenceFeature, ReferenceTopicInput, ReferenceTopicModelConfig, argmax,
        bounded_count, build_result, dot, expectation, next_unit, normalize, objective,
        require_finite, standardize_event_time,
    };
    use crate::TopicMeasurementError;
    use temporal_core::EventTime;
    use uuid::Uuid;

    #[test]
    fn numeric_helpers_are_deterministic_and_fail_closed() {
        let mut seed = 1;
        let first = next_unit(&mut seed);
        assert!(first > 0.0);
        assert!(first < 1.0);
        assert!((dot(&[1.0, 2.0], &[3.0, 4.0]) - 11.0).abs() < f64::EPSILON);
        assert_eq!(argmax(&[0.1, 0.8, 0.1]), 1);
        assert_eq!(argmax(&[]), 0);
        let mut values = [1.0, 3.0];
        normalize(&mut values).expect("normalize");
        assert!((values[0] - 0.25).abs() < f64::EPSILON);
        assert!((values[1] - 0.75).abs() < f64::EPSILON);
        assert_eq!(
            normalize(&mut [0.0, 0.0]),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        assert_eq!(
            normalize(&mut [f64::INFINITY]),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        assert_eq!(require_finite(1.0), Ok(()));
        assert_eq!(
            require_finite(f64::NAN),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
    }

    #[test]
    fn impossible_numeric_states_and_mixed_topic_edges_fail_closed() {
        let input = ReferenceTopicInput {
            document_ids: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
            term_rows: vec![vec![(0, f64::MAX)], vec![(0, f64::MAX)]],
            vocabulary_size: 2,
            design: vec![vec![1.0], vec![1.0]],
            features: vec![PrevalenceFeature::Intercept],
            transition_pairs: vec![(0, 1)],
        };
        let theta = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let zero_beta = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        assert_eq!(
            expectation(&input, &theta, &zero_beta, 2),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        let infinite_beta = vec![vec![f64::INFINITY, 0.0], vec![0.0, 0.0]];
        assert_eq!(
            expectation(&input, &theta, &infinite_beta, 2),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        let finite_beta = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        assert_eq!(
            expectation(&input, &theta, &finite_beta, 2),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        let finite_input = ReferenceTopicInput {
            term_rows: vec![vec![(0, 1.0)], vec![(0, 1.0)]],
            ..input.clone()
        };
        assert!(expectation(&finite_input, &theta, &finite_beta, 2).is_ok());

        let config = ReferenceTopicModelConfig::new(2, vec![1], 10, 1e-6).expect("config");
        assert_eq!(
            objective(
                &input,
                &config,
                &theta,
                &[vec![0.0], vec![0.0]],
                &[vec![0.0], vec![0.0]],
                &[vec![0.0]],
                f64::INFINITY,
            ),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        assert!(
            objective(
                &input,
                &config,
                &theta,
                &[vec![0.0], vec![0.0]],
                &[vec![0.0], vec![0.0]],
                &[vec![0.0]],
                0.0,
            )
            .is_ok()
        );

        let result = build_result(
            &input,
            &config,
            FitState {
                seed: 1,
                iterations: 4,
                objective: -1.0,
                beta: finite_beta,
                eta: vec![vec![10.0], vec![-10.0]],
                coefficients: vec![vec![0.0]],
            },
        )
        .expect("mixed-topic result");
        assert!(result.sequence_edges.is_empty());
        assert_eq!(result.connected_post_count, 0);
        assert_eq!(result.lineage_count, 0);

        let same_time = EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("time");
        assert_eq!(
            standardize_event_time(&[same_time, same_time]),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            bounded_count(usize::try_from(u64::from(u32::MAX) + 1).expect("wide usize")),
            Err(TopicMeasurementError::InvalidModelInput)
        );
    }
}
