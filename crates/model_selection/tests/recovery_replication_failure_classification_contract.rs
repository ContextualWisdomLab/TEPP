use model_selection::{ModelSelectionError, admit_recovery_replication_result};

#[test]
fn only_numerical_recovery_failures_enter_the_failed_replication_denominator() {
    assert_eq!(admit_recovery_replication_result(Ok(3_u32)), Ok(Some(3)));

    for error in [
        ModelSelectionError::RecoveryCandidateFitFailed,
        ModelSelectionError::NoSuccessfulFit,
        ModelSelectionError::InvalidDiagnostic,
    ] {
        assert_eq!(admit_recovery_replication_result::<u32>(Err(error)), Ok(None));
    }

    for error in [
        ModelSelectionError::NonPositiveCandidateK,
        ModelSelectionError::EmptyCandidateSet,
        ModelSelectionError::LlmVoteIsNotStatisticalAuthority,
        ModelSelectionError::LexicalWeightForbidden,
        ModelSelectionError::InsufficientRecoveryReplications,
        ModelSelectionError::PartitionInputMismatch,
        ModelSelectionError::DuplicateCandidateK,
        ModelSelectionError::RollingOriginWindowMismatch,
        ModelSelectionError::RepeatedEvaluationDocument,
        ModelSelectionError::PredictiveCandidateSetMismatch,
        ModelSelectionError::PredictiveCandidateGridMismatch,
        ModelSelectionError::PredictiveCandidateConfigurationMismatch,
        ModelSelectionError::PredictiveCandidateTrainingStateMismatch,
    ] {
        assert_eq!(
            admit_recovery_replication_result::<u32>(Err(error)),
            Err(error),
            "structural recovery invalidity must invalidate the experiment"
        );
    }
}
