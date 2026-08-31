//! Fit each candidate `K` and score it from the actual reference model.

use std::collections::BTreeSet;

use topic_measurement::{
    ReferenceTopicInput, ReferenceTopicModel, ReferenceTopicModelConfig, fit_reference_topic_model,
    refuse_lexical_inferential_weight,
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

    /// Build the exact reference-estimator configuration for one candidate.
    ///
    /// # Errors
    ///
    /// Returns [`ModelSelectionError::NonPositiveCandidateK`] when `candidate_k`
    /// is not in this validated candidate set, or
    /// [`ModelSelectionError::InvalidDiagnostic`] when conversion fails.
    pub fn reference_config(
        &self,
        candidate_k: u32,
    ) -> Result<ReferenceTopicModelConfig, ModelSelectionError> {
        if !self.candidate_topic_counts.contains(&candidate_k) {
            return Err(ModelSelectionError::NonPositiveCandidateK);
        }
        let topic_count =
            usize::try_from(candidate_k).map_err(|_| ModelSelectionError::InvalidDiagnostic)?;
        ReferenceTopicModelConfig::new(
            topic_count,
            self.seeds.clone(),
            self.maximum_iterations,
            self.tolerance,
        )
        .and_then(|value| {
            value.with_hyperparameters(
                self.prior_variance,
                self.relation_strength,
                self.ridge,
                self.topic_smoothing,
                self.step_size,
            )
        })
        .map_err(|_| ModelSelectionError::InvalidDiagnostic)
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
        .map(|(candidate_k, _)| candidate_k)
}

/// Fit each candidate and return the selected `K` with its exact fitted model.
///
/// # Errors
///
/// Returns the same typed failures as [`select_fitted_candidate_k`].
pub fn select_fitted_candidate_model(
    input: &ReferenceTopicInput,
    config: &FittedCandidateKConfig,
    method_name: &str,
    llm_votes: &[u32],
) -> Result<(u32, ReferenceTopicModel), ModelSelectionError> {
    refuse_nonstatistical_method(method_name)?;
    let mut candidates = Vec::new();
    let mut fitted = Vec::new();
    for &candidate_k in config.candidate_topic_counts() {
        let fit_config = config.reference_config(candidate_k)?;
        if let Ok(model) = fit_reference_topic_model(input, &fit_config) {
            candidates.push(statistical_candidate_from_fit(input, candidate_k, &model)?);
            fitted.push((candidate_k, model));
        }
    }
    for &vote in llm_votes {
        candidates.push(ModelCandidate::llm_vote_only(vote)?);
    }
    if candidates.is_empty() {
        return Err(ModelSelectionError::NoSuccessfulFit);
    }
    let selected_k = select_candidate_k(&candidates)?;
    fitted
        .into_iter()
        .find(|(candidate_k, _)| *candidate_k == selected_k)
        .ok_or(ModelSelectionError::NoSuccessfulFit)
}

fn refuse_nonstatistical_method(method: &str) -> Result<(), ModelSelectionError> {
    refuse_lexical_inferential_weight(method)
        .map_err(|_| ModelSelectionError::LexicalWeightForbidden)?;
    let folded: String = method
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    if matches!(
        folded.as_str(),
        "stopword"
            | "stopwords"
            | "stopworddeletion"
            | "llm"
            | "llmlabel"
            | "llmlabels"
            | "llmvote"
            | "llmvoteonly"
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
    use super::{FittedCandidateKConfig, free_parameter_count, refuse_nonstatistical_method};
    use crate::ModelSelectionError;
    use topic_measurement::{PrevalenceFeature, ReferenceTopicModel};

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
        assert!(config.reference_config(2).is_ok());
        assert_eq!(
            config.reference_config(4),
            Err(ModelSelectionError::NonPositiveCandidateK)
        );
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
