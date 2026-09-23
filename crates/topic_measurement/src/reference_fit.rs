//! Owner-issued binding of admitted reference inputs, configurations, and fits.
//!
//! `ReferenceTopicModel` remains a numerical result type whose fields are useful
//! for scientific inspection and hostile-case testing. A release projection,
//! however, must not attach that result to a different dimension-compatible
//! input after fitting. `ReferenceTopicFit` removes that substitution path by
//! retaining private clones of the exact admitted input and configuration beside
//! the model returned by the reference estimator.
//!
//! Held-out evaluation has an additional coordinate requirement: fitted
//! prevalence coefficients must stay paired with the frozen training prevalence
//! basis that defined them. `ReferenceTopicTrainingFit` composes the exact
//! `ReferenceTopicTrainingInput` with the `ReferenceTopicFit` minted from that
//! same input; no public constructor accepts pre-existing detached components.
//!
//! These are fit-local numerical integrity contracts, not Evidence
//! authentication. They do not prove the external source snapshot, vocabulary
//! lineage, Membership provenance, relation activation, or availability clock.

use membership_core::MembershipNetwork;
use temporal_core::EventTime;
use uuid::Uuid;

use crate::{
    JointCoordinatePrecision, PrevalenceDesignBasis, ReferenceTopicInput, ReferenceTopicModel,
    ReferenceTopicModelConfig, ReferenceTopicTrainingInput, SparseMatrix, TopicMeasurementError,
    fit_reference_topic_model, from_additive_log_ratio,
};

/// Owner-issued nominal aggregate for one admitted CPU reference fit.
#[derive(Clone, Debug)]
pub struct ReferenceTopicFit {
    input: ReferenceTopicInput,
    config: ReferenceTopicModelConfig,
    model: ReferenceTopicModel,
}

impl ReferenceTopicFit {
    /// Fit the deterministic CPU reference estimator and retain its exact input.
    ///
    /// No public constructor accepts a pre-existing model, so callers cannot
    /// pair model quantities from one fit with document/design coordinates from
    /// another fit while preserving this owner-issued type.
    ///
    /// # Errors
    ///
    /// Propagates the reference estimator's invalid-input, non-finite, and
    /// convergence failures. No aggregate is returned for a partial fit.
    pub fn fit(
        input: &ReferenceTopicInput,
        config: &ReferenceTopicModelConfig,
    ) -> Result<Self, TopicMeasurementError> {
        let model = fit_reference_topic_model(input, config)?;
        Ok(Self {
            input: input.clone(),
            config: config.clone(),
            model,
        })
    }

    /// Return the exact admitted estimator input retained for this fit.
    #[must_use]
    pub const fn input(&self) -> &ReferenceTopicInput {
        &self.input
    }

    /// Return the exact deterministic configuration retained for this fit.
    #[must_use]
    pub const fn config(&self) -> &ReferenceTopicModelConfig {
        &self.config
    }

    /// Return the model produced from the retained input and configuration.
    #[must_use]
    pub const fn model(&self) -> &ReferenceTopicModel {
        &self.model
    }

    /// Build the joint generalized-Gauss-Newton precision for this exact fit.
    ///
    /// The retained input, configuration, and numerical model are consumed as
    /// one owner-issued aggregate, preventing a caller from rebinding a model
    /// from one fit to dimension-compatible input/configuration from another.
    /// `topic_ids` remain provisional caller-supplied coordinates until the
    /// Evidence-owned vocabulary provenance and released topic-basis contract
    /// are settled; this method does not promote them to semantic authority.
    ///
    /// The detached input/model/config builder is intentionally unavailable to
    /// downstream consumers. This compile-fail contract prevents the owner
    /// aggregate from being bypassed through the input type:
    ///
    /// ```compile_fail
    /// use topic_measurement::ReferenceTopicInput;
    /// let _ = ReferenceTopicInput::build_joint_coordinate_precision;
    /// ```
    ///
    /// # Errors
    ///
    /// Propagates invalid dimension, identity, non-finite, symmetry, or
    /// positive-definiteness failures from the joint-precision owner.
    pub fn build_joint_coordinate_precision(
        &self,
        topic_ids: Vec<Uuid>,
    ) -> Result<JointCoordinatePrecision, TopicMeasurementError> {
        self.input
            .build_joint_coordinate_precision(&self.model, &self.config, topic_ids)
    }
}

/// Owner-issued fitted training state for leakage-safe prevalence projection.
///
/// This aggregate retains the exact [`ReferenceTopicTrainingInput`] that minted
/// the frozen prevalence basis together with the [`ReferenceTopicFit`] produced
/// from that input. Downstream predictive evaluation can therefore consume one
/// nominal owner value instead of independently pairing a fit and a
/// dimension-compatible basis from different training states.
#[derive(Clone, Debug)]
pub struct ReferenceTopicTrainingFit {
    training_input: ReferenceTopicTrainingInput,
    reference_fit: ReferenceTopicFit,
}

impl ReferenceTopicTrainingFit {
    /// Fit one owner-issued training input and retain its frozen prevalence basis.
    ///
    /// No public constructor accepts an existing [`ReferenceTopicFit`] or
    /// [`PrevalenceDesignBasis`], so detached A-fit/B-basis substitution cannot
    /// preserve this owner-issued type.
    ///
    /// # Errors
    ///
    /// Propagates the CPU reference estimator's invalid-input, non-finite, and
    /// convergence failures. No aggregate is returned for a partial fit.
    pub fn fit(
        training_input: &ReferenceTopicTrainingInput,
        config: &ReferenceTopicModelConfig,
    ) -> Result<Self, TopicMeasurementError> {
        let reference_fit = ReferenceTopicFit::fit(training_input.input(), config)?;
        Ok(Self {
            training_input: training_input.clone(),
            reference_fit,
        })
    }

    /// Return the exact admitted training input and frozen prevalence basis owner.
    #[must_use]
    pub const fn training_input(&self) -> &ReferenceTopicTrainingInput {
        &self.training_input
    }

    /// Return the frozen prevalence coordinate system used by the training input.
    #[must_use]
    pub const fn prevalence_design_basis(&self) -> &PrevalenceDesignBasis {
        self.training_input.prevalence_design_basis()
    }

    /// Return the reference fit minted from this exact training input.
    #[must_use]
    pub const fn reference_fit(&self) -> &ReferenceTopicFit {
        &self.reference_fit
    }

    /// Score evaluation counts under the fitted training prevalence mean.
    ///
    /// Evaluation EventTime, covariates, and Membership are projected through
    /// the frozen training [`PrevalenceDesignBasis`]. The retained prevalence
    /// coefficients then produce one ALR mean per evaluation row, which is
    /// transformed to topic proportions and scored against the retained training
    /// topic-term probabilities. No evaluation count updates topic-term
    /// probabilities, prevalence coefficients, training document coordinates,
    /// or relation parameters.
    ///
    /// The returned vector is one log likelihood per evaluation document. This
    /// quantity is deliberately named a prevalence-mean predictive score: it is
    /// not STM document-completion likelihood, not the in-sample Schwarz score,
    /// and not evidence that a rolling-origin cutoff was respected. Cutoff and
    /// source admission remain owner responsibilities outside this numerical
    /// primitive.
    ///
    /// The private training-fit fields make fitted coefficient/basis/topic-term
    /// dimensional agreement an owner invariant. Evaluation-controlled geometry
    /// and counts are still validated here and fail closed.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] when evaluation row
    /// geometry, vocabulary width, count support, or frozen-feature coordinates
    /// are incompatible. Returns [`TopicMeasurementError::NonFiniteEstimate`]
    /// when finite evaluation counts overflow the log-likelihood accumulation;
    /// ALR conversion errors propagate unchanged.
    pub fn prevalence_mean_predictive_log_likelihoods(
        &self,
        document_ids: &[Uuid],
        document_term: &SparseMatrix,
        event_times: &[EventTime],
        covariates: Option<&SparseMatrix>,
        memberships: &MembershipNetwork,
    ) -> Result<Vec<f64>, TopicMeasurementError> {
        let basis = self.prevalence_design_basis();
        let model = self.reference_fit.model();
        let topic_count = model.topic_term_probabilities.len();
        let coordinate_count = topic_count - 1;
        let vocabulary_size = model.topic_term_probabilities[0].len();
        if document_term.rows() != document_ids.len()
            || document_term.columns() != vocabulary_size
        {
            return Err(TopicMeasurementError::InvalidModelInput);
        }

        let design = basis.project(document_ids, event_times, covariates, memberships)?;
        let term_rows = document_term.row_entries();
        let mut scores = Vec::with_capacity(document_ids.len());
        for (row_index, terms) in term_rows.iter().enumerate() {
            let row_total = terms.iter().map(|(_, count)| count).sum::<f64>();
            if terms.is_empty()
                || terms.iter().any(|(_, count)| *count < 0.0)
                || !row_total.is_finite()
                || row_total <= 0.0
            {
                return Err(TopicMeasurementError::InvalidModelInput);
            }

            let mut alr_mean = vec![0.0; coordinate_count];
            for (feature, value) in design[row_index].iter().copied().enumerate() {
                for (coordinate, coefficient) in model.prevalence_coefficients[feature]
                    .iter()
                    .copied()
                    .enumerate()
                {
                    alr_mean[coordinate] += value * coefficient;
                }
            }
            let theta = from_additive_log_ratio(&alr_mean)?;
            let mut log_likelihood = 0.0_f64;
            for &(term, count) in terms {
                let probability = (0..topic_count)
                    .map(|topic| theta[topic] * model.topic_term_probabilities[topic][term])
                    .sum::<f64>();
                log_likelihood += count * probability.ln();
            }
            if !log_likelihood.is_finite() {
                return Err(TopicMeasurementError::NonFiniteEstimate);
            }
            scores.push(log_likelihood);
        }
        Ok(scores)
    }

    /// Infer evaluation-document topic states while keeping the training fit frozen.
    ///
    /// Evaluation EventTime, covariates, and Membership are projected through the
    /// exact frozen [`PrevalenceDesignBasis`] retained by this fit. Each evaluation
    /// ALR coordinate starts at that projected prevalence mean and is then optimized
    /// independently against only its own observed term counts and the fitted
    /// Gaussian prevalence prior. Topic-term probabilities, prevalence coefficients,
    /// training document coordinates, relation parameters, and all other global
    /// training state remain unchanged.
    ///
    /// This is a deterministic local MAP-style coordinate recovery primitive for
    /// scientific held-out state validation. It is deliberately separate from
    /// [`Self::prevalence_mean_predictive_log_likelihoods`]: the latter evaluates
    /// counts at the prevalence mean, whereas this method lets the held-out counts
    /// update only their own local latent coordinate. No held-out relation graph is
    /// admitted here, so adding another evaluation row cannot change the focal row.
    /// It is not posterior sampling and does not by itself establish calibrated
    /// interval coverage or rolling-origin source admission.
    ///
    /// Returned simplex rows follow `document_ids` exactly.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] for incompatible row,
    /// vocabulary, count, or frozen-design geometry. Returns
    /// [`TopicMeasurementError::NonFiniteEstimate`] for non-finite numerical state
    /// and [`TopicMeasurementError::DidNotConverge`] when the retained deterministic
    /// iteration budget is exhausted. ALR conversion errors propagate unchanged.
    pub fn infer_held_out_document_topic_proportions(
        &self,
        document_ids: &[Uuid],
        document_term: &SparseMatrix,
        event_times: &[EventTime],
        covariates: Option<&SparseMatrix>,
        memberships: &MembershipNetwork,
    ) -> Result<Vec<Vec<f64>>, TopicMeasurementError> {
        let model = self.reference_fit.model();
        let config = self.reference_fit.config();
        let topic_count = model.topic_term_probabilities.len();
        let coordinate_count = topic_count - 1;
        let vocabulary_size = model.topic_term_probabilities[0].len();
        if document_ids.is_empty()
            || document_term.rows() != document_ids.len()
            || document_term.columns() != vocabulary_size
            || event_times.len() != document_ids.len()
        {
            return Err(TopicMeasurementError::InvalidModelInput);
        }

        let design = self
            .prevalence_design_basis()
            .project(document_ids, event_times, covariates, memberships)?;
        let term_rows = document_term.row_entries();
        let mut recovered = Vec::with_capacity(document_ids.len());
        for (row_index, terms) in term_rows.iter().enumerate() {
            let row_total = terms.iter().map(|(_, count)| count).sum::<f64>();
            if terms.is_empty()
                || terms
                    .iter()
                    .any(|(_, count)| !count.is_finite() || *count < 0.0)
                || !row_total.is_finite()
                || row_total <= 0.0
            {
                return Err(TopicMeasurementError::InvalidModelInput);
            }
            let mean = projected_prevalence_mean(
                &design[row_index],
                &model.prevalence_coefficients,
                coordinate_count,
            )?;
            recovered.push(infer_local_document_state(
                terms,
                &model.topic_term_probabilities,
                &mean,
                row_total,
                config,
            )?);
        }
        Ok(recovered)
    }
}

fn projected_prevalence_mean(
    design_row: &[f64],
    coefficients: &[Vec<f64>],
    coordinate_count: usize,
) -> Result<Vec<f64>, TopicMeasurementError> {
    if design_row.len() != coefficients.len()
        || coefficients.is_empty()
        || design_row.iter().any(|value| !value.is_finite())
        || coefficients.iter().any(|row| {
            row.len() != coordinate_count || row.iter().any(|value| !value.is_finite())
        })
    {
        return Err(TopicMeasurementError::InvalidModelInput);
    }
    let mut mean = vec![0.0; coordinate_count];
    for (feature, value) in design_row.iter().copied().enumerate() {
        for (coordinate, coefficient) in coefficients[feature].iter().copied().enumerate() {
            mean[coordinate] += value * coefficient;
        }
    }
    if mean.iter().any(|value| !value.is_finite()) {
        Err(TopicMeasurementError::NonFiniteEstimate)
    } else {
        Ok(mean)
    }
}

fn infer_local_document_state(
    terms: &[(usize, f64)],
    topic_term_probabilities: &[Vec<f64>],
    mean: &[f64],
    token_count: f64,
    config: &ReferenceTopicModelConfig,
) -> Result<Vec<f64>, TopicMeasurementError> {
    let topic_count = topic_term_probabilities.len();
    let coordinate_count = topic_count - 1;
    let mut eta = mean.to_vec();
    let mut previous_objective = None;

    for iteration in 1..=config.maximum_iterations() {
        let theta = from_additive_log_ratio(&eta)?;
        let mut expected_topic_counts = vec![0.0; topic_count];
        let mut log_likelihood = 0.0_f64;
        for &(term, count) in terms {
            let probability = (0..topic_count)
                .map(|topic| theta[topic] * topic_term_probabilities[topic][term])
                .sum::<f64>();
            if !probability.is_finite() || probability <= 0.0 {
                return Err(TopicMeasurementError::NonFiniteEstimate);
            }
            log_likelihood += count * probability.ln();
            for topic in 0..topic_count {
                expected_topic_counts[topic] +=
                    count * theta[topic] * topic_term_probabilities[topic][term] / probability;
            }
        }
        let prior = eta
            .iter()
            .zip(mean)
            .map(|(value, center)| (value - center).powi(2))
            .sum::<f64>()
            / (2.0 * config.prior_variance());
        let objective = log_likelihood - prior;
        if !objective.is_finite() {
            return Err(TopicMeasurementError::NonFiniteEstimate);
        }
        if iteration > 3
            && previous_objective.is_some_and(|previous: f64| {
                (objective - previous).abs() / (1.0 + previous.abs()) <= config.tolerance()
            })
        {
            return Ok(theta);
        }
        previous_objective = Some(objective);

        let scale = config.step_size() / (1.0 + token_count);
        for coordinate in 0..coordinate_count {
            let gradient = expected_topic_counts[coordinate]
                - token_count * theta[coordinate]
                - (eta[coordinate] - mean[coordinate]) / config.prior_variance();
            eta[coordinate] += scale * gradient;
        }
    }
    Err(TopicMeasurementError::DidNotConverge)
}

#[cfg(test)]
mod tests {
    use super::{infer_local_document_state, projected_prevalence_mean};
    use crate::{ReferenceTopicModelConfig, TopicMeasurementError};

    fn config(maximum_iterations: usize, tolerance: f64) -> ReferenceTopicModelConfig {
        ReferenceTopicModelConfig::new(2, vec![7], maximum_iterations, tolerance)
            .and_then(|value| value.with_hyperparameters(1.0, 0.0, 0.0, 0.05, 0.2))
            .expect("local-state test configuration")
    }

    #[test]
    fn projected_mean_rejects_invalid_geometry_and_overflow() {
        assert_eq!(
            projected_prevalence_mean(&[1.0], &[], 1),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        assert_eq!(
            projected_prevalence_mean(&[1.0, 2.0], &[vec![0.0]], 1),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        assert_eq!(
            projected_prevalence_mean(&[f64::NAN], &[vec![0.0]], 1),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        assert_eq!(
            projected_prevalence_mean(&[1.0], &[vec![0.0, 1.0]], 1),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        assert_eq!(
            projected_prevalence_mean(&[1.0], &[vec![f64::NAN]], 1),
            Err(TopicMeasurementError::InvalidModelInput)
        );
        assert_eq!(
            projected_prevalence_mean(
                &[f64::MAX, f64::MAX],
                &[vec![1.0], vec![1.0]],
                1,
            ),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        assert_eq!(
            projected_prevalence_mean(&[1.0, 2.0], &[vec![3.0], vec![4.0]], 1),
            Ok(vec![11.0])
        );
    }

    #[test]
    fn local_state_fails_closed_on_numerical_failure_and_iteration_exhaustion() {
        let terms = [(0, 1.0)];
        let mean = [0.0];
        assert_eq!(
            infer_local_document_state(
                &terms,
                &[vec![0.0, 1.0], vec![0.0, 1.0]],
                &mean,
                1.0,
                &config(10, 1.0e-6),
            ),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        assert_eq!(
            infer_local_document_state(
                &terms,
                &[vec![f64::INFINITY, 0.0], vec![0.1, 0.9]],
                &mean,
                1.0,
                &config(10, 1.0e-6),
            ),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        assert_eq!(
            infer_local_document_state(
                &[(0, f64::MAX)],
                &[vec![0.1, 0.9], vec![0.1, 0.9]],
                &mean,
                f64::MAX,
                &config(10, 1.0e-6),
            ),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
        assert_eq!(
            infer_local_document_state(
                &terms,
                &[vec![0.9, 0.1], vec![0.1, 0.9]],
                &mean,
                1.0,
                &config(2, 1.0e-12),
            ),
            Err(TopicMeasurementError::DidNotConverge)
        );
        let recovered = infer_local_document_state(
            &[(0, 9.0), (1, 1.0)],
            &[vec![0.9, 0.1], vec![0.1, 0.9]],
            &mean,
            10.0,
            &config(2_000, 1.0e-6),
        )
        .expect("converged local state");
        assert_eq!(recovered.len(), 2);
        assert!(recovered[0] > recovered[1]);
    }
}
