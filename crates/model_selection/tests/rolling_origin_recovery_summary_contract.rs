use model_selection::{ModelSelectionError, selected_k_recovery_summary};

#[test]
fn recovery_summary_preserves_failure_denominator_and_monte_carlo_uncertainty() {
    let summary = selected_k_recovery_summary(&[Some(3), None, Some(5), Some(4), None], 4)
        .expect("recovery summary");

    assert_eq!(summary.truth_k(), 4);
    assert_eq!(summary.replication_count(), 5);
    assert_eq!(summary.success_count(), 3);
    assert_eq!(summary.failure_count(), 2);
    assert!((summary.failure_rate() - 0.4).abs() < 1e-12);
    assert_eq!(summary.bias(), Some(0.0));
    assert!(
        (summary
            .root_mean_square_error()
            .expect("conditional rmse")
            - (2.0_f64 / 3.0).sqrt())
        .abs()
            < 1e-12
    );
    assert!(
        (summary
            .bias_monte_carlo_standard_error()
            .expect("bias mcse")
            - (1.0_f64 / 3.0).sqrt())
        .abs()
            < 1e-12
    );
    assert!(
        (summary
            .rmse_monte_carlo_standard_error()
            .expect("rmse mcse")
            - 0.204_124_145_231_931_5)
            .abs()
            < 1e-12
    );
}

#[test]
fn perfect_recovery_has_zero_monte_carlo_error_but_keeps_failures() {
    let summary = selected_k_recovery_summary(&[Some(4), Some(4), None, Some(4)], 4)
        .expect("perfect conditional recovery");

    assert_eq!(summary.replication_count(), 4);
    assert_eq!(summary.success_count(), 3);
    assert_eq!(summary.failure_count(), 1);
    assert_eq!(summary.bias(), Some(0.0));
    assert_eq!(summary.root_mean_square_error(), Some(0.0));
    assert_eq!(summary.bias_monte_carlo_standard_error(), Some(0.0));
    assert_eq!(summary.rmse_monte_carlo_standard_error(), Some(0.0));
}

#[test]
fn recovery_summary_preserves_failure_evidence_with_limited_success_support() {
    assert_eq!(
        selected_k_recovery_summary(&[], 4),
        Err(ModelSelectionError::EmptyCandidateSet)
    );

    let all_failed = selected_k_recovery_summary(&[None, None], 4)
        .expect("all-failed experiment remains reportable");
    assert_eq!(all_failed.success_count(), 0);
    assert_eq!(all_failed.failure_count(), 2);
    assert_eq!(all_failed.failure_rate(), 1.0);
    assert_eq!(all_failed.bias(), None);
    assert_eq!(all_failed.root_mean_square_error(), None);
    assert_eq!(all_failed.bias_monte_carlo_standard_error(), None);
    assert_eq!(all_failed.rmse_monte_carlo_standard_error(), None);

    let one_success = selected_k_recovery_summary(&[Some(5), None], 4)
        .expect("one-success experiment remains reportable");
    assert_eq!(one_success.success_count(), 1);
    assert_eq!(one_success.failure_count(), 1);
    assert_eq!(one_success.bias(), Some(1.0));
    assert_eq!(one_success.root_mean_square_error(), Some(1.0));
    assert_eq!(one_success.bias_monte_carlo_standard_error(), None);
    assert_eq!(one_success.rmse_monte_carlo_standard_error(), None);

    assert_eq!(
        selected_k_recovery_summary(&[Some(1), Some(4)], 4),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        selected_k_recovery_summary(&[Some(4), Some(4)], 1),
        Err(ModelSelectionError::NonPositiveCandidateK)
    );
}
