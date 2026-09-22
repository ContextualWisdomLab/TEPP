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

use uuid::Uuid;

use crate::{
    JointCoordinatePrecision, PrevalenceDesignBasis, ReferenceTopicInput, ReferenceTopicModel,
    ReferenceTopicModelConfig, ReferenceTopicTrainingInput, TopicMeasurementError,
    fit_reference_topic_model,
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
}
