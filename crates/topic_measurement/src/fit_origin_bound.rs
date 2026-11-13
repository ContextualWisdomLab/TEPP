//! Additive private-proposal route; legacy DTO precision remains unbound.

use super::{
    JointCoordinatePrecision, ReferenceTopicInput, ReferenceTopicModel, ReferenceTopicModelConfig,
    fit_reference_topic_model,
};
use crate::TopicMeasurementError;
use uuid::Uuid;

/// An immutable actual CPU fit paired with the exact admitted numerical input.
///
/// This additive proposal does not make the legacy public model DTO or its
/// precision entry safe. Callers must explicitly migrate to this route.
#[derive(Clone, Debug)]
pub struct FitOriginBoundModel {
    input: ReferenceTopicInput,
    config: ReferenceTopicModelConfig,
    model: ReferenceTopicModel,
}

/// Run the existing CPU f64 estimator and retain its actual admitted origin.
///
/// The input is moved, not cloned; estimator equations and convergence remain
/// unchanged. The small configuration is retained with the resulting model.
///
/// # Errors
/// Propagates the existing estimator's admission, finite and convergence errors.
pub fn fit_origin_bound_topic_model(
    input: ReferenceTopicInput,
    config: ReferenceTopicModelConfig,
) -> Result<FitOriginBoundModel, TopicMeasurementError> {
    let model = fit_reference_topic_model(&input, &config)?;
    Ok(FitOriginBoundModel {
        input,
        config,
        model,
    })
}

impl FitOriginBoundModel {
    /// Inspect the fitted result without obtaining mutable access to its owner.
    #[must_use]
    pub const fn model(&self) -> &ReferenceTopicModel {
        &self.model
    }

    /// Score the actual originating CPU fit on its owned admitted counts.
    ///
    /// Delegates unchanged to the CPU f64 in-sample mixture likelihood, not
    /// the penalized objective or a held-out score. No foreign input or model
    /// can be supplied. This does not attest external cutoff authority or GPU work.
    ///
    /// # Errors
    /// Propagates the original scorer's dimension and non-finite errors.
    pub fn in_sample_log_likelihood(&self) -> Result<f64, TopicMeasurementError> {
        self.input.in_sample_log_likelihood(&self.model)
    }

    /// Build precision only for the actual fitted input and configuration.
    ///
    /// Equality covers ordered documents/times, sparse numerical rows,
    /// vocabulary, expanded design/features and modeled transition pairs.
    /// Configuration equality includes both numerical and execution controls.
    /// This validates admitted numerical identity, not external run/snapshot/
    /// cutoff authority or completeness of a posterior artifact. Topic identity
    /// still has the legacy explicit caller-assigned basis semantics.
    ///
    /// # Errors
    /// Returns invalid input on any origin mismatch, otherwise propagates the
    /// unchanged precision builder's errors.
    pub fn build_joint_coordinate_precision(
        &self,
        input: &ReferenceTopicInput,
        config: &ReferenceTopicModelConfig,
        topic_ids: Vec<Uuid>,
    ) -> Result<JointCoordinatePrecision, TopicMeasurementError> {
        let original = &self.input;
        if config != &self.config
            || input.document_ids != original.document_ids
            || input.event_times != original.event_times
            || input.term_rows != original.term_rows
            || input.vocabulary_size != original.vocabulary_size
            || input.design != original.design
            || input.features != original.features
            || input.transition_pairs != original.transition_pairs
        {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        original.build_joint_coordinate_precision(&self.model, &self.config, topic_ids)
    }

    /// Build precision from the retained actual fitted input and configuration.
    ///
    /// Use this after moving the input and configuration into
    /// [`fit_origin_bound_topic_model`]; neither value needs to be cloned or
    /// supplied again. Delegates unchanged to the CPU `f64` precision builder.
    /// It does not attest external run/snapshot/cutoff authority, GPU work or
    /// posterior-artifact completeness, and topic identity remains caller-assigned.
    ///
    /// # Errors
    /// Propagates the unchanged precision builder's dimension, topic-identity,
    /// non-finite, symmetry and positive-definiteness errors.
    pub fn build_owned_joint_coordinate_precision(
        &self,
        topic_ids: Vec<Uuid>,
    ) -> Result<JointCoordinatePrecision, TopicMeasurementError> {
        self.input
            .build_joint_coordinate_precision(&self.model, &self.config, topic_ids)
    }
}

#[cfg(test)]
#[path = "origin_guard_cases.rs"]
mod origin_guard_cases;
