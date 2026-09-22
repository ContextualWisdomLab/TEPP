//! Owner-issued binding of an admitted reference input, configuration, and fit.
//!
//! `ReferenceTopicModel` remains a numerical result type whose fields are useful
//! for scientific inspection and hostile-case testing. A release projection,
//! however, must not attach that result to a different dimension-compatible
//! input after fitting. `ReferenceTopicFit` removes that substitution path by
//! retaining private clones of the exact admitted input and configuration beside
//! the model returned by the reference estimator.
//!
//! This is fit-local integrity, not Evidence authentication. It does not prove
//! the external source snapshot, vocabulary lineage, or availability clock.

use crate::{
    ReferenceTopicInput, ReferenceTopicModel, ReferenceTopicModelConfig, TopicMeasurementError,
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
}
