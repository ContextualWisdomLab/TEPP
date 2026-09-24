//! Rolling-origin identity binding for held-out document-state recovery.
//!
//! `topic_measurement` owns the local latent-state arithmetic. This module owns
//! only the scientific-recovery ACL that binds those numerical rows to an
//! admitted rolling-origin partition and classifies numerical recovery failure
//! separately from structural evaluation invalidity.

use std::collections::BTreeSet;

use corpus_split::RollingOriginPartition;
use membership_core::MembershipNetwork;
use temporal_core::EventTime;
use topic_measurement::{ReferenceTopicTrainingFit, SparseMatrix, TopicMeasurementError};
use uuid::Uuid;

use crate::ModelSelectionError;

fn held_out_state_result<T>(
    result: Result<T, TopicMeasurementError>,
) -> Result<T, ModelSelectionError> {
    result.map_err(|error| match error {
        TopicMeasurementError::DidNotConverge
        | TopicMeasurementError::NonFiniteEstimate
        | TopicMeasurementError::InvalidLogRatioDimension => {
            ModelSelectionError::HeldOutStateRecoveryFailed
        }
        _ => ModelSelectionError::HeldOutStateEvaluationInputInvalid,
    })
}

/// Infer held-out document states only for the exact admitted rolling-origin rows.
///
/// The retained training identities of `training_fit` must equal the partition's
/// training set. Evaluation identities must be duplicate-free and equal the
/// partition's evaluation set before any local-state arithmetic runs. The method
/// then delegates to
/// [`ReferenceTopicTrainingFit::infer_held_out_document_topic_proportions`], so
/// topic-term probabilities, prevalence coefficients, relation parameters,
/// training document states, and training input remain frozen.
///
/// This function does not authenticate Evidence, EventTime, Membership, or
/// relation provenance and does not admit a held-out relation graph. It binds
/// already-owned split identities to the #719 numerical owner.
///
/// # Errors
///
/// Returns [`ModelSelectionError::PartitionInputMismatch`] when the retained
/// training identities or evaluation identities do not exactly match the
/// admitted partition. Returns
/// [`ModelSelectionError::HeldOutStateRecoveryFailed`] for non-convergence,
/// non-finite numerical state, or ALR representability failure reached during
/// local optimization. Returns
/// [`ModelSelectionError::HeldOutStateEvaluationInputInvalid`] for structural
/// topic-measurement failures, including incompatible row/vocabulary/count/design
/// geometry. Unknown future topic-measurement errors fail closed as structural
/// invalidity until explicitly classified.
pub fn rolling_origin_held_out_document_topic_proportions(
    partition: &RollingOriginPartition,
    training_fit: &ReferenceTopicTrainingFit,
    evaluation_document_ids: &[Uuid],
    evaluation_document_term: &SparseMatrix,
    evaluation_event_times: &[EventTime],
    evaluation_covariates: Option<&SparseMatrix>,
    memberships: &MembershipNetwork,
) -> Result<Vec<Vec<f64>>, ModelSelectionError> {
    let fitted_training_ids: BTreeSet<_> = training_fit
        .reference_fit()
        .input()
        .document_ids()
        .iter()
        .copied()
        .collect();
    if fitted_training_ids != *partition.training_document_ids() {
        return Err(ModelSelectionError::PartitionInputMismatch);
    }

    let evaluation_ids: BTreeSet<_> = evaluation_document_ids.iter().copied().collect();
    if evaluation_ids.len() != evaluation_document_ids.len()
        || evaluation_ids != *partition.evaluation_document_ids()
    {
        return Err(ModelSelectionError::PartitionInputMismatch);
    }

    held_out_state_result(training_fit.infer_held_out_document_topic_proportions(
        evaluation_document_ids,
        evaluation_document_term,
        evaluation_event_times,
        evaluation_covariates,
        memberships,
    ))
}

#[cfg(test)]
mod tests {
    use super::held_out_state_result;
    use crate::ModelSelectionError;
    use topic_measurement::TopicMeasurementError;

    #[test]
    fn held_out_state_failure_classification_is_fail_closed() {
        for error in [
            TopicMeasurementError::DidNotConverge,
            TopicMeasurementError::NonFiniteEstimate,
            TopicMeasurementError::InvalidLogRatioDimension,
        ] {
            assert_eq!(
                held_out_state_result::<()>(Err(error)),
                Err(ModelSelectionError::HeldOutStateRecoveryFailed)
            );
        }
        for error in [
            TopicMeasurementError::InvalidModelInput,
            TopicMeasurementError::InvalidSparseMatrix,
            TopicMeasurementError::InvalidComposition,
            TopicMeasurementError::LexicalWeightForbidden,
            TopicMeasurementError::JointPosteriorUnavailable,
        ] {
            assert_eq!(
                held_out_state_result::<()>(Err(error)),
                Err(ModelSelectionError::HeldOutStateEvaluationInputInvalid)
            );
        }
        assert_eq!(held_out_state_result(Ok(7_u8)), Ok(7_u8));
    }
}
