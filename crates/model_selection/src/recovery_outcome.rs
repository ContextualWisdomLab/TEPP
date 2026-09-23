//! Failure classification for one scientific recovery replication.
//!
//! Recovery summaries must retain numerical fit/selection failures in the
//! attempted denominator without turning structural experiment invalidity into
//! an ordinary failed replication. This module owns that distinction so repeated
//! recovery harnesses do not reimplement it with `.ok()` or caller-local error
//! whitelists.

use crate::ModelSelectionError;

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

#[cfg(test)]
mod tests {
    use super::admit_recovery_replication_result;
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
}
