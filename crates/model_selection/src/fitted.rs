//! Fit each candidate `K` and score it from the actual reference model.

use std::collections::BTreeSet;

use topic_measurement::{
    MAX_REFERENCE_FIT_BUDGET, MAX_REFERENCE_WORKING_CELLS, ReferenceTopicInput,
    ReferenceTopicModel, ReferenceTopicModelConfig, TopicMeasurementError,
    fit_reference_topic_model, refuse_lexical_inferential_weight,
};

use crate::{ModelCandidate, ModelSelectionError, select_candidate_k};

/// ADR 0012 `σ²` prior variance. Identical to `topic_measurement` reference.
const DEFAULT_PRIOR_VARIANCE: f64 = 1.0;
/// ADR 0012 network penalty `λ`. Identical to `topic_measurement` reference.
const DEFAULT_RELATION_STRENGTH: f64 = 0.25;
/// ADR 0012 coefficient ridge `ρ`. Identical to `topic_measurement` reference.
const DEFAULT_RIDGE: f64 = 0.01;
/// Smoothed multinomial `β` floor. Identical to `topic_measurement` reference.
const DEFAULT_TOPIC_SMOOTHING: f64 = 0.05;
/// Bounded GEM step. Identical to `topic_measurement` reference.
const DEFAULT_STEP_SIZE: f64 = 0.2;

/// Compact outcome retained for one requested candidate topic count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FittedCandidateOutcome {
    candidate_k: u32,
    statistical_score: Option<f64>,
    complexity: Option<f64>,
    failure: Option<TopicMeasurementError>,
    seed: Option<u64>,
    iterations: Option<usize>,
    objective: Option<f64>,
}

impl FittedCandidateOutcome {
    /// Return the requested topic count.
    #[must_use]
    pub const fn candidate_k(self) -> u32 {
        self.candidate_k
    }

    /// Return the finite Schwarz score for a successful fit.
    #[must_use]
    pub const fn statistical_score(self) -> Option<f64> {
        self.statistical_score
    }

    /// Return the finite free-parameter count for a successful fit.
    #[must_use]
    pub const fn complexity(self) -> Option<f64> {
        self.complexity
    }

    /// Return the typed estimator failure for an unsuccessful fit.
    #[must_use]
    pub const fn failure(self) -> Option<TopicMeasurementError> {
        self.failure
    }

    /// Return the stable artifact code for an unsuccessful fit.
    #[must_use]
    pub const fn failure_code(self) -> Option<&'static str> {
        match self.failure {
            Some(TopicMeasurementError::InvalidModelInput) => Some("invalid_model_input"),
            Some(TopicMeasurementError::NonFiniteEstimate) => Some("non_finite_estimate"),
            Some(TopicMeasurementError::DidNotConverge) => Some("did_not_converge"),
            Some(_) => Some("unsupported_estimator_failure"),
            None => None,
        }
    }

    /// Return the converged seed, iteration count, and objective for a successful fit.
    #[must_use]
    pub const fn fit_receipt(self) -> Option<(u64, usize, f64)> {
        match (self.seed, self.iterations, self.objective) {
            (Some(seed), Some(iterations), Some(objective)) => Some((seed, iterations, objective)),
            _ => None,
        }
    }
}

/// Winning fitted model plus bounded, reason-bearing selection evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct FittedCandidateSelection {
    selected_k: u32,
    model: ReferenceTopicModel,
    candidate_outcomes: Vec<FittedCandidateOutcome>,
    llm_recommendations: Vec<u32>,
}

/// Typed selection failure retaining every completed candidate outcome.
#[derive(Clone, Debug, PartialEq)]
pub struct FittedCandidateSelectionFailure {
    error: ModelSelectionError,
    candidate_outcomes: Vec<FittedCandidateOutcome>,
    llm_recommendations: Vec<u32>,
}

impl FittedCandidateSelectionFailure {
    /// Return the governing model-selection error.
    #[must_use]
    pub const fn error(&self) -> ModelSelectionError {
        self.error
    }

    /// Return all candidate outcomes completed before failure.
    #[must_use]
    pub fn candidate_outcomes(&self) -> &[FittedCandidateOutcome] {
        &self.candidate_outcomes
    }

    /// Return separated non-authoritative LLM recommendations.
    #[must_use]
    pub fn llm_recommendations(&self) -> &[u32] {
        &self.llm_recommendations
    }
}

impl std::fmt::Display for FittedCandidateSelectionFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for FittedCandidateSelectionFailure {}

impl FittedCandidateSelection {
    /// Return the statistically selected topic count.
    #[must_use]
    pub const fn selected_k(&self) -> u32 {
        self.selected_k
    }

    /// Return the canonical statistical selection method.
    #[must_use]
    pub const fn method_code(&self) -> &'static str {
        "trsl_tm_reference_bic_v1"
    }

    /// Return all requested candidate outcomes in caller order.
    #[must_use]
    pub fn candidate_outcomes(&self) -> &[FittedCandidateOutcome] {
        &self.candidate_outcomes
    }

    /// Return separated non-authoritative LLM recommendations.
    #[must_use]
    pub fn llm_recommendations(&self) -> &[u32] {
        &self.llm_recommendations
    }

    /// Consume the receipt and return the already-selected fitted model.
    #[must_use]
    pub fn into_model(self) -> ReferenceTopicModel {
        self.model
    }
}

/// Seeds, iteration budget, and candidate topic counts for fitted selection.
#[derive(Clone, Debug, PartialEq)]
pub struct FittedCandidateKConfig {
    candidate_topic_counts: Vec<u32>,
    seeds: Vec<u64>,
    maximum_iterations: usize,
    tolerance: f64,
    prior_variance: f64,
    relation_strength: f64,
    ridge: f64,
    topic_smoothing: f64,
    step_size: f64,
}

impl FittedCandidateKConfig {
    /// Construct a fitted-selection configuration with ADR-owned defaults.
    ///
    /// # Errors
    ///
    /// Returns [`ModelSelectionError::EmptyCandidateSet`] when no candidate
    /// `K` is supplied, [`ModelSelectionError::NonPositiveCandidateK`] when any
    /// candidate is less than two, or
    /// [`ModelSelectionError::InvalidDiagnostic`] when candidates are
    /// duplicated or the seed/iteration/tolerance contract fails.
    pub fn new(
        candidate_topic_counts: Vec<u32>,
        seeds: Vec<u64>,
        maximum_iterations: usize,
        tolerance: f64,
    ) -> Result<Self, ModelSelectionError> {
        let value = Self {
            candidate_topic_counts,
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

    /// Replace numerical hyperparameters while retaining candidate `K` and seeds.
    ///
    /// # Errors
    ///
    /// Returns [`ModelSelectionError::InvalidDiagnostic`] for any non-finite or
    /// non-positive value, except `relation_strength` and `ridge`, which may be
    /// exactly zero for a declared ablation.
    pub fn with_hyperparameters(
        mut self,
        prior_variance: f64,
        relation_strength: f64,
        ridge: f64,
        topic_smoothing: f64,
        step_size: f64,
    ) -> Result<Self, ModelSelectionError> {
        self.prior_variance = prior_variance;
        self.relation_strength = relation_strength;
        self.ridge = ridge;
        self.topic_smoothing = topic_smoothing;
        self.step_size = step_size;
        self.validate()?;
        Ok(self)
    }

    /// Return the candidate topic counts in caller order.
    #[must_use]
    pub fn candidate_topic_counts(&self) -> &[u32] {
        &self.candidate_topic_counts
    }

    /// Return the estimator initialization seeds.
    #[must_use]
    pub fn seeds(&self) -> &[u64] {
        &self.seeds
    }

    /// Return the per-seed iteration budget.
    #[must_use]
    pub const fn maximum_iterations(&self) -> usize {
        self.maximum_iterations
    }

    /// Return the relative-objective convergence tolerance.
    #[must_use]
    pub const fn tolerance(&self) -> f64 {
        self.tolerance
    }

    /// Return prior, relation, ridge, smoothing, and step controls in that order.
    #[must_use]
    pub const fn hyperparameters(&self) -> [f64; 5] {
        [
            self.prior_variance,
            self.relation_strength,
            self.ridge,
            self.topic_smoothing,
            self.step_size,
        ]
    }

    fn validate(&self) -> Result<(), ModelSelectionError> {
        if self.candidate_topic_counts.is_empty() {
            return Err(ModelSelectionError::EmptyCandidateSet);
        }
        let mut seen = BTreeSet::new();
        for &candidate_k in &self.candidate_topic_counts {
            if candidate_k < 2 {
                return Err(ModelSelectionError::NonPositiveCandidateK);
            }
            if !seen.insert(candidate_k) {
                return Err(ModelSelectionError::InvalidDiagnostic);
            }
        }
        if self.seeds.is_empty()
            || self.candidate_topic_counts.len() > MAX_REFERENCE_FIT_BUDGET
            || self.seeds.len() > MAX_REFERENCE_FIT_BUDGET
            || self.maximum_iterations < 2
            || self.maximum_iterations > MAX_REFERENCE_FIT_BUDGET
            || self
                .candidate_topic_counts
                .len()
                .checked_mul(self.seeds.len())
                .and_then(|fits| fits.checked_mul(self.maximum_iterations))
                .is_none_or(|work| work > MAX_REFERENCE_WORKING_CELLS)
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
            return Err(ModelSelectionError::InvalidDiagnostic);
        }
        Ok(())
    }
}

/// Build a statistically supported candidate from one actual fitted model.
///
/// The first diagnostic is Schwarz's (1978) large-sample maximizer
/// `ℓ − (p ln N)/2` from the fitted `θ` and `β`. Complexity is the free
/// parameter count `p`. A failed or non-finite diagnostic is returned as a
/// typed error; it is never replaced with a fabricated likelihood.
///
/// # Errors
///
/// Returns [`ModelSelectionError::NonPositiveCandidateK`] when `candidate_k`
/// is less than two, or [`ModelSelectionError::InvalidDiagnostic`] when the
/// fitted dimensions, likelihood, or parameter count are unusable.
pub fn statistical_candidate_from_fit(
    input: &ReferenceTopicInput,
    candidate_k: u32,
    model: &ReferenceTopicModel,
) -> Result<ModelCandidate, ModelSelectionError> {
    let log_likelihood = input
        .in_sample_log_likelihood(model)
        .map_err(|_| ModelSelectionError::InvalidDiagnostic)?;
    let tokens = input
        .token_count()
        .map_err(|_| ModelSelectionError::InvalidDiagnostic)?;
    let parameters = free_parameter_count(model)?;
    let log_tokens = tokens.ln();
    if log_tokens < 0.0 {
        return Err(ModelSelectionError::InvalidDiagnostic);
    }
    let penalty = parameters * log_tokens;
    let score = log_likelihood - 0.5 * penalty;
    ModelCandidate::statistical(candidate_k, score, parameters)
}

/// Fit each candidate `K` and select the admissible statistical topic count.
///
/// Each candidate is fitted with [`fit_reference_topic_model`]. A typed
/// `DidNotConverge`, `NonFiniteEstimate`, or `InvalidModelInput` is a failed
/// candidate, not a fabricated diagnostic. LLM-vote-only values may be
/// supplied as recommenders; they cannot win without a successful statistical
/// fit. TF-IDF, BM25, stopword-deletion, and LLM labels are refused as
/// inferential coordinates.
///
/// This wiring does not claim GPU execution, full Bayesian sampling, or topic
/// birth/split/merge.
///
/// # Errors
///
/// Returns a typed model-selection failure when the method is lexical, the
/// configuration is invalid, every fit fails, or only LLM votes remain.
pub fn select_fitted_candidate_k(
    input: &ReferenceTopicInput,
    config: &FittedCandidateKConfig,
    method_name: &str,
    llm_votes: &[u32],
) -> Result<u32, ModelSelectionError> {
    select_fitted_candidate_model(input, config, method_name, llm_votes)
        .map(|selection| selection.selected_k())
        .map_err(|failure| failure.error())
}

/// Fit every candidate `K` and return the selected converged CPU `f64` model.
///
/// This retains the winning fit so downstream execution does not refit it.
/// LLM-only votes remain non-statistical recommenders and cannot supply the
/// returned model.
///
/// # Errors
///
/// Returns the same typed failures as [`select_fitted_candidate_k`].
///
/// # Panics
///
/// Panics only if the already-validated fitted-selection configuration cannot
/// construct its identical reference-estimator configuration, or a converged
/// reference fit violates its own finite-diagnostic contract.
#[allow(clippy::too_many_lines)]
pub fn select_fitted_candidate_model(
    input: &ReferenceTopicInput,
    config: &FittedCandidateKConfig,
    method_name: &str,
    llm_votes: &[u32],
) -> Result<FittedCandidateSelection, FittedCandidateSelectionFailure> {
    refuse_nonstatistical_method(method_name)
        .map_err(|error| selection_failure(error, Vec::new(), llm_votes))?;
    if config.candidate_topic_counts().len() > input.vocabulary_size().saturating_sub(1)
        || llm_votes.len() > MAX_REFERENCE_FIT_BUDGET
    {
        return Err(selection_failure(
            ModelSelectionError::InvalidDiagnostic,
            Vec::new(),
            llm_votes,
        ));
    }
    let mut candidates = Vec::new();
    let mut candidate_outcomes = Vec::with_capacity(config.candidate_topic_counts().len());
    let mut selected_model = None;
    for &candidate_k in config.candidate_topic_counts() {
        #[allow(clippy::cast_possible_truncation)]
        let topic_count = candidate_k as usize;
        let fit_config = ReferenceTopicModelConfig::new(
            topic_count,
            config.seeds().to_vec(),
            config.maximum_iterations(),
            config.tolerance(),
        )
        .and_then(|value| {
            value.with_hyperparameters(
                config.prior_variance,
                config.relation_strength,
                config.ridge,
                config.topic_smoothing,
                config.step_size,
            )
        })
        .expect("validated fitted config maps to the identical reference config");
        match fit_reference_topic_model(input, &fit_config) {
            Ok(model) => {
                let candidate = statistical_candidate_from_fit(input, candidate_k, &model)
                    .map_err(|error| {
                        selection_failure(error, candidate_outcomes.clone(), llm_votes)
                    })?;
                candidate_outcomes.push(FittedCandidateOutcome {
                    candidate_k,
                    statistical_score: candidate.held_out_log_likelihood(),
                    complexity: candidate.complexity(),
                    failure: None,
                    seed: Some(model.seed),
                    iterations: Some(model.iterations),
                    objective: Some(model.objective),
                });
                candidates.push(candidate);
                if select_candidate_k(&candidates)
                    .expect("a statistical candidate always has a selected K")
                    == candidate_k
                {
                    selected_model = Some((candidate_k, model));
                }
            }
            Err(failure) => candidate_outcomes.push(FittedCandidateOutcome {
                candidate_k,
                statistical_score: None,
                complexity: None,
                failure: Some(failure),
                seed: None,
                iterations: None,
                objective: None,
            }),
        }
    }
    for &vote in llm_votes {
        candidates
            .push(ModelCandidate::llm_vote_only(vote).map_err(|error| {
                selection_failure(error, candidate_outcomes.clone(), llm_votes)
            })?);
    }
    if candidates.is_empty() {
        return Err(selection_failure(
            ModelSelectionError::NoSuccessfulFit,
            candidate_outcomes,
            llm_votes,
        ));
    }
    let selected_k = select_candidate_k(&candidates)
        .map_err(|error| selection_failure(error, candidate_outcomes.clone(), llm_votes))?;
    let model = selected_model
        .filter(|(candidate_k, _)| *candidate_k == selected_k)
        .map(|(_, model)| model)
        .ok_or_else(|| {
            selection_failure(
                ModelSelectionError::LlmVoteIsNotStatisticalAuthority,
                candidate_outcomes.clone(),
                llm_votes,
            )
        })?;
    Ok(FittedCandidateSelection {
        selected_k,
        model,
        candidate_outcomes,
        llm_recommendations: llm_votes.to_vec(),
    })
}

fn selection_failure(
    error: ModelSelectionError,
    candidate_outcomes: Vec<FittedCandidateOutcome>,
    llm_recommendations: &[u32],
) -> FittedCandidateSelectionFailure {
    FittedCandidateSelectionFailure {
        error,
        candidate_outcomes,
        llm_recommendations: llm_recommendations.to_vec(),
    }
}

fn refuse_nonstatistical_method(method: &str) -> Result<(), ModelSelectionError> {
    refuse_lexical_inferential_weight(method)
        .map_err(|_| ModelSelectionError::LexicalWeightForbidden)?;
    if !matches!(
        method,
        "trsl_tm_reference" | "tepp_topic_measurement" | "logistic_normal"
    ) {
        return Err(ModelSelectionError::LexicalWeightForbidden);
    }
    Ok(())
}

fn free_parameter_count(model: &ReferenceTopicModel) -> Result<f64, ModelSelectionError> {
    let topic_count = model.document_topic_proportions.first().map_or(0, Vec::len);
    let vocabulary = model.topic_term_probabilities.first().map_or(0, Vec::len);
    let documents = model.document_topic_proportions.len();
    let features = model.prevalence_coefficients.len();
    if topic_count < 2
        || vocabulary < 2
        || documents < 2
        || model.topic_term_probabilities.len() != topic_count
        || model
            .topic_term_probabilities
            .iter()
            .any(|row| row.len() != vocabulary)
        || model
            .document_topic_proportions
            .iter()
            .any(|row| row.len() != topic_count)
        || model
            .prevalence_coefficients
            .iter()
            .any(|row| row.len() != topic_count - 1)
    {
        return Err(ModelSelectionError::InvalidDiagnostic);
    }
    Ok(topic_count as f64 * (vocabulary - 1) as f64
        + (documents as f64 + features as f64) * (topic_count - 1) as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        FittedCandidateKConfig, FittedCandidateOutcome, free_parameter_count,
        refuse_nonstatistical_method,
    };
    use crate::ModelSelectionError;
    use topic_measurement::{PrevalenceFeature, ReferenceTopicModel, TopicMeasurementError};

    #[test]
    fn fitted_failure_codes_are_stable() {
        for (failure, code) in [
            (
                TopicMeasurementError::InvalidModelInput,
                "invalid_model_input",
            ),
            (
                TopicMeasurementError::NonFiniteEstimate,
                "non_finite_estimate",
            ),
            (TopicMeasurementError::DidNotConverge, "did_not_converge"),
            (
                TopicMeasurementError::InvalidComposition,
                "unsupported_estimator_failure",
            ),
        ] {
            let outcome = FittedCandidateOutcome {
                candidate_k: 2,
                statistical_score: None,
                complexity: None,
                failure: Some(failure),
                seed: None,
                iterations: None,
                objective: None,
            };
            assert_eq!(outcome.failure_code(), Some(code));
        }
        let successful = FittedCandidateOutcome {
            candidate_k: 2,
            statistical_score: Some(-1.0),
            complexity: Some(2.0),
            failure: None,
            seed: Some(1),
            iterations: Some(2),
            objective: Some(-1.0),
        };
        assert_eq!(successful.failure_code(), None);
        assert_eq!(successful.fit_receipt(), Some((1, 2, -1.0)));
    }

    fn model(
        topic_term_probabilities: Vec<Vec<f64>>,
        document_topic_proportions: Vec<Vec<f64>>,
        prevalence_coefficients: Vec<Vec<f64>>,
    ) -> ReferenceTopicModel {
        ReferenceTopicModel {
            seed: 1,
            iterations: 4,
            objective: -1.0,
            topic_term_probabilities,
            document_topic_proportions,
            document_coordinate_variances: Vec::new(),
            prevalence_coefficients,
            prevalence_features: vec![PrevalenceFeature::Intercept],
            sequence_edges: Vec::new(),
            connected_post_count: 0,
            lineage_count: 0,
        }
    }

    #[test]
    fn configuration_accessors_and_method_gate_cover_local_branches() {
        let config =
            FittedCandidateKConfig::new(vec![2, 3], vec![7, 11], 20, 1e-5).expect("config");
        assert_eq!(config.candidate_topic_counts(), &[2, 3]);
        assert_eq!(config.seeds(), &[7, 11]);
        assert_eq!(config.maximum_iterations(), 20);
        assert!((config.tolerance() - 1e-5).abs() < f64::EPSILON);
        refuse_nonstatistical_method("trsl_tm_reference").expect("allowed");
        refuse_nonstatistical_method("logistic_normal").expect("allowed");
        for method in [
            "tfidf",
            "bm25",
            "keyword",
            "stopword",
            "stopwords",
            "stopworddeletion",
            "llm",
            "llmlabel",
            "llm-labels",
            "llm_vote",
            "llm_vote_only",
        ] {
            assert_eq!(
                refuse_nonstatistical_method(method),
                Err(ModelSelectionError::LexicalWeightForbidden)
            );
        }
        assert_eq!(
            FittedCandidateKConfig::new(vec![2], vec![1], 10, 1e-6)
                .expect("base")
                .with_hyperparameters(1.0, 0.0, 0.0, 0.05, 0.2)
                .expect("zero ablation")
                .seeds()
                .len(),
            1
        );
    }

    #[test]
    fn free_parameter_count_refuses_dimension_mismatch() {
        let valid = model(
            vec![vec![0.7, 0.3], vec![0.2, 0.8]],
            vec![vec![0.9, 0.1], vec![0.1, 0.9]],
            vec![vec![0.0]],
        );
        assert!((free_parameter_count(&valid).expect("p") - 5.0).abs() < f64::EPSILON);
        assert_eq!(
            free_parameter_count(&model(Vec::new(), Vec::new(), Vec::new())),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            free_parameter_count(&model(
                vec![vec![0.5, 0.5], vec![0.5, 0.5]],
                vec![vec![0.5, 0.5]],
                vec![vec![0.0]]
            )),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            free_parameter_count(&model(
                vec![vec![0.5, 0.5]],
                vec![vec![0.5, 0.5], vec![0.5, 0.5]],
                vec![vec![0.0]]
            )),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            free_parameter_count(&model(
                vec![vec![1.0], vec![1.0]],
                vec![vec![0.5, 0.5], vec![0.5, 0.5]],
                vec![vec![0.0]]
            )),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            free_parameter_count(&model(
                vec![vec![1.0, 0.0]],
                vec![vec![1.0], vec![1.0]],
                vec![vec![]]
            )),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            free_parameter_count(&model(
                vec![vec![0.5, 0.5], vec![0.5, 0.5, 0.0]],
                vec![vec![0.5, 0.5], vec![0.5, 0.5]],
                vec![vec![0.0]]
            )),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            free_parameter_count(&model(
                vec![vec![0.5, 0.5], vec![0.5, 0.5]],
                vec![vec![0.5, 0.5], vec![0.5, 0.5, 0.0]],
                vec![vec![0.0]]
            )),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert_eq!(
            free_parameter_count(&model(
                vec![vec![0.5, 0.5], vec![0.5, 0.5]],
                vec![vec![0.5, 0.5], vec![0.5, 0.5]],
                vec![vec![0.0, 0.0]]
            )),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
    }
}
