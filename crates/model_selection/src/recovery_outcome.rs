//! Failure classification for one scientific recovery replication.
//!
//! Recovery summaries must retain numerical fit/selection failures in the
//! attempted denominator without turning structural experiment invalidity into
//! an ordinary failed replication. This module owns that distinction so repeated
//! recovery harnesses do not reimplement it with `.ok()` or caller-local error
//! whitelists.

use crate::{ModelSelectionError, SelectedKRecoverySummary, selected_k_recovery_summary};

/// Admit one scientific-recovery operation into the replication denominator.
///
/// Successful values become `Some(value)`. Failures caused by estimator fitting
/// or unusable numerical model-selection diagnostics become `None`, preserving
/// the attempted replication without fabricating a selected result. All other
/// errors propagate because they describe an invalid candidate design, split,
/// identity, chronology, authority boundary, or recovery composition rather than
/// stochastic/numerical replication failure.
///
/// The function is generic so the same owner classification can be applied to
/// complete-grid fitting and to the subsequent selected-`K` result. It must not
/// be used to suppress errors from unrelated workflows.
///
/// # Errors
///
/// Returns every [`ModelSelectionError`] except
/// [`ModelSelectionError::RecoveryCandidateFitFailed`],
/// [`ModelSelectionError::NoSuccessfulFit`], and
/// [`ModelSelectionError::InvalidDiagnostic`], which are represented as a failed
/// recovery replication (`Ok(None)`).
pub fn admit_recovery_replication_result<T>(
    result: Result<T, ModelSelectionError>,
) -> Result<Option<T>, ModelSelectionError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(
            ModelSelectionError::RecoveryCandidateFitFailed
            | ModelSelectionError::NoSuccessfulFit
            | ModelSelectionError::InvalidDiagnostic,
        ) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Summarize typed selected-`K` recovery results without caller-authored `None` values.
///
/// This is the canonical scientific composition of recovery failure admission and
/// denominator-preserving summary. Each typed replication result is first passed
/// through [`admit_recovery_replication_result`]. Only numerical fit/selection
/// failures therefore become failed replications; any structural split, identity,
/// horizon, configuration, training-state, predictive-input, or authority error
/// aborts the experiment before a summary can be minted.
///
/// The lower-level [`selected_k_recovery_summary`] remains available for callers
/// that already hold owner-admitted `Option<u32>` outcomes, but #680-style
/// repeated scientific recovery should prefer this function so structural errors
/// cannot be converted to `None` with caller-local `.ok()` handling.
///
/// # Errors
///
/// Propagates structural [`ModelSelectionError`] values unchanged, then delegates
/// summary validation to [`selected_k_recovery_summary`], including empty
/// experiments, invalid truth `K`, or invalid successful selected `K` values.
pub fn selected_k_recovery_summary_from_results<I>(
    results: I,
    truth_k: u32,
) -> Result<SelectedKRecoverySummary, ModelSelectionError>
where
    I: IntoIterator<Item = Result<u32, ModelSelectionError>>,
{
    let admitted: Result<Vec<_>, _> = results
        .into_iter()
        .map(admit_recovery_replication_result)
        .collect();
    selected_k_recovery_summary(&admitted?, truth_k)
}

#[cfg(test)]
mod tests {
    use super::{admit_recovery_replication_result, selected_k_recovery_summary_from_results};
    use crate::ModelSelectionError;

    #[test]
    fn owner_keeps_numerical_failure_separate_from_structural_invalidity() {
        assert_eq!(admit_recovery_replication_result(Ok(4_u32)), Ok(Some(4)));
        assert_eq!(
            admit_recovery_replication_result::<u32>(Err(
                ModelSelectionError::RecoveryCandidateFitFailed
            )),
            Ok(None)
        );
        assert_eq!(
            admit_recovery_replication_result::<u32>(Err(
                ModelSelectionError::PartitionInputMismatch
            )),
            Err(ModelSelectionError::PartitionInputMismatch)
        );
    }

    #[test]
    fn typed_summary_composes_owner_admission_before_statistics() {
        let summary = selected_k_recovery_summary_from_results(
            [
                Ok(3),
                Err(ModelSelectionError::RecoveryCandidateFitFailed),
                Ok(5),
            ],
            4,
        )
        .expect("typed recovery summary");
        assert_eq!(summary.replication_count(), 3);
        assert_eq!(summary.success_count(), 2);
        assert_eq!(summary.failure_count(), 1);
        assert_eq!(summary.bias(), Some(0.0));
        assert_eq!(
            selected_k_recovery_summary_from_results(
                [Ok(4), Err(ModelSelectionError::RecoveryWindowSetMismatch)],
                4,
            ),
            Err(ModelSelectionError::RecoveryWindowSetMismatch)
        );
    }
}
