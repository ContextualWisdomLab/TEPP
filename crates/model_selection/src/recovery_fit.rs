//! Complete-grid fitting for scientific selected-K recovery.
//!
//! Operational model selection may retain survivor semantics when some candidate
//! fits fail. Scientific recovery cannot: a missing declared K changes the
//! experimental design and must count as a failed replication. This module owns
//! the public path from one [`FittedCandidateKConfig`] to every corresponding
//! [`ReferenceTopicTrainingFit`] without requiring consumers to reconstruct the
//! numerical configuration.

use topic_measurement::{
    ReferenceTopicTrainingFit, ReferenceTopicTrainingInput, TopicMeasurementError,
};

use crate::{FittedCandidateKConfig, ModelSelectionError};

fn recovery_fit_result<T>(
    result: Result<T, TopicMeasurementError>,
) -> Result<T, ModelSelectionError> {
    result.map_err(|error| match error {
        TopicMeasurementError::DidNotConverge
        | TopicMeasurementError::NonFiniteEstimate
        | TopicMeasurementError::InvalidLogRatioDimension => {
            ModelSelectionError::RecoveryCandidateFitFailed
        }
        _ => ModelSelectionError::RecoveryCandidateInputInvalid,
    })
}

/// Fit every candidate in one predeclared scientific recovery design.
///
/// Candidates are returned in the declaration order of
/// [`FittedCandidateKConfig::candidate_topic_counts`]. Each estimator
/// configuration is minted through the configuration owner's canonical
/// conversion, so seeds, convergence controls, and hyperparameters cannot drift
/// through a second consumer-side construction path. If any candidate fails
/// numerically, no survivor subset is returned and the caller may record the
/// replication as a numerical failure. A deterministic input/configuration or
/// authority failure invalidates the recovery experiment instead of entering the
/// failed-replication denominator.
///
/// Generic operational selection deliberately keeps its existing survivor
/// semantics. This function is only the stricter scientific-recovery path.
///
/// # Errors
///
/// Returns the configuration owner's typed validation failure when a declared
/// candidate cannot be converted to the reference configuration,
/// [`ModelSelectionError::RecoveryCandidateFitFailed`] for non-convergence,
/// non-finite arithmetic, or ALR representability failure reached while fitting,
/// or [`ModelSelectionError::RecoveryCandidateInputInvalid`] for structural
/// topic-measurement failures. Unknown future topic-measurement failures fail
/// closed as structural invalidity until explicitly classified.
pub fn fit_declared_recovery_candidates(
    training_input: &ReferenceTopicTrainingInput,
    config: &FittedCandidateKConfig,
) -> Result<Vec<ReferenceTopicTrainingFit>, ModelSelectionError> {
    let mut fits = Vec::with_capacity(config.candidate_topic_counts().len());
    for &candidate_k in config.candidate_topic_counts() {
        let fit_config = config.reference_topic_model_config(candidate_k)?;
        let fit = recovery_fit_result(ReferenceTopicTrainingFit::fit(
            training_input,
            &fit_config,
        ))?;
        fits.push(fit);
    }
    Ok(fits)
}

#[cfg(test)]
mod tests {
    use super::recovery_fit_result;
    use crate::ModelSelectionError;
    use topic_measurement::TopicMeasurementError;

    #[test]
    fn numerical_failures_enter_the_recovery_failure_path() {
        for error in [
            TopicMeasurementError::DidNotConverge,
            TopicMeasurementError::NonFiniteEstimate,
            TopicMeasurementError::InvalidLogRatioDimension,
        ] {
            assert_eq!(
                recovery_fit_result::<()>(Err(error)),
                Err(ModelSelectionError::RecoveryCandidateFitFailed)
            );
        }
        assert_eq!(recovery_fit_result(Ok(7_u8)), Ok(7_u8));
    }

    #[test]
    fn structural_topic_failures_invalidate_the_recovery_experiment() {
        for error in [
            TopicMeasurementError::InvalidModelInput,
            TopicMeasurementError::InvalidSparseMatrix,
            TopicMeasurementError::InvalidComposition,
            TopicMeasurementError::LexicalWeightForbidden,
            TopicMeasurementError::JointPosteriorUnavailable,
        ] {
            assert_eq!(
                recovery_fit_result::<()>(Err(error)),
                Err(ModelSelectionError::RecoveryCandidateInputInvalid)
            );
        }
    }
}
