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
}
