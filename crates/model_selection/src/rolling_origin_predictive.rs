//! Consumer binding between admitted rolling-origin partitions and predictive rows.

use std::collections::BTreeSet;

use corpus_split::RollingOriginPartition;
use membership_core::MembershipNetwork;
use temporal_core::EventTime;
use topic_measurement::{ReferenceTopicTrainingFit, SparseMatrix};
use uuid::Uuid;

use crate::ModelSelectionError;

/// Score one admitted rolling-origin evaluation partition under a fixed training fit.
///
/// The training fit's exact document identity set must match the admitted
/// partition's training set, and the caller-order evaluation identities must be
/// duplicate-free and equal the partition's evaluation set. This prevents a
/// valid cutoff/leakage receipt for one partition from being rebound to
/// dimension-compatible numerical rows from another partition.
///
/// The numerical quantity is the sum of #684's per-document fixed-training
/// prevalence-mean predictive log likelihoods. It is not STM document-completion
/// likelihood, the in-sample Schwarz score, or a new split/cutoff authority.
///
/// # Errors
///
/// Returns [`ModelSelectionError::PartitionInputMismatch`] when training or
/// evaluation identities do not exactly match the admitted partition. Returns
/// [`ModelSelectionError::InvalidDiagnostic`] when the topic-measurement owner
/// rejects evaluation geometry/coordinates/counts or when finite per-document
/// scores overflow while aggregating the partition diagnostic.
pub fn rolling_origin_prevalence_mean_predictive_log_likelihood(
    partition: &RollingOriginPartition,
    training_fit: &ReferenceTopicTrainingFit,
    evaluation_document_ids: &[Uuid],
    evaluation_document_term: &SparseMatrix,
    evaluation_event_times: &[EventTime],
    evaluation_covariates: Option<&SparseMatrix>,
    memberships: &MembershipNetwork,
) -> Result<f64, ModelSelectionError> {
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

    let scores = training_fit
        .prevalence_mean_predictive_log_likelihoods(
            evaluation_document_ids,
            evaluation_document_term,
            evaluation_event_times,
            evaluation_covariates,
            memberships,
        )
        .map_err(|_| ModelSelectionError::InvalidDiagnostic)?;
    let diagnostic = scores.into_iter().sum::<f64>();
    if !diagnostic.is_finite() {
        return Err(ModelSelectionError::InvalidDiagnostic);
    }
    Ok(diagnostic)
}
