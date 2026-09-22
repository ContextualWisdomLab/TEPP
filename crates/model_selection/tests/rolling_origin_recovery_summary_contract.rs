use model_selection::{
    ModelSelectionError, selected_k_recovery_summary,
};

#[test]
fn recovery_summary_preserves_failure_denominator_and_monte_carlo_uncertainty() {
    let summary = selected_k_recovery_summary(&[Some(3), None, Some(5), Some(4), None], 4)
        .expect("recovery summary");

    assert_eq!(summary.truth_k(), 4);
    assert_eq!(summary.replication_count(), 5);
    assert_eq!(summary.success_count(), 3);
    assert_eq!(summary.failure_count(), 2);
    assert!((summary.failure_rate() - 0.4).abs() < 1e-12);
    assert!(summary.bias().abs() < 1e-12);
    assert!((summary.root_mean_square_error() - (2.0_f64 / 3.0).sqrt()).abs() < 1e-12);
    assert!((summary.bias_monte_carlo_standard_error() - (1.0_f64 / 3.0).sqrt()).abs() < 1e-12);
    assert!((summary.rmse_monte_carlo_standard_error() - 0.204_124_145_231_931_5).abs() < 1e-12);
}

#[test]
fn recovery_summary_fails_closed_without_replication_support() {
    assert_eq!(
        selected_k_recovery_summary(&[Some(4), None], 4),
        Err(ModelSelectionError::InsufficientRecoveryReplications)
    );
    assert_eq!(
        selected_k_recovery_summary(&[Some(1), Some(4)], 4),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        selected_k_recovery_summary(&[Some(4), Some(4)], 1),
        Err(ModelSelectionError::NonPositiveCandidateK)
    );
}
