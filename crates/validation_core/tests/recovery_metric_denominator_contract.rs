use validation_core::{
    ValidationError, summarize_recovery_metric_replications,
};

#[test]
fn recovery_metric_summary_preserves_attempted_failure_denominator() {
    let summary = summarize_recovery_metric_replications(4, &[0.10, 0.20], 0.025, 0.975)
        .expect("mixed recovery summary");

    assert_eq!(summary.attempted_replication_count(), 4);
    assert_eq!(summary.successful_replication_count(), 2);
    assert_eq!(summary.failure_count(), 2);
    assert!((summary.failure_rate() - 0.5).abs() < 1.0e-12);
    assert!((summary.failure_rate_standard_error() - 0.25).abs() < 1.0e-12);

    let successful = summary
        .successful_metric_summary()
        .expect("successful metric summary");
    assert_eq!(successful.replication_count, 2);
    assert!((successful.mean - 0.15).abs() < 1.0e-12);
}

#[test]
fn recovery_metric_summary_retains_catastrophic_failure_without_fabricating_metric() {
    let summary = summarize_recovery_metric_replications(3, &[], 0.025, 0.975)
        .expect("all-failed recovery summary");

    assert_eq!(summary.attempted_replication_count(), 3);
    assert_eq!(summary.successful_replication_count(), 0);
    assert_eq!(summary.failure_count(), 3);
    assert!((summary.failure_rate() - 1.0).abs() < 1.0e-12);
    assert_eq!(summary.failure_rate_standard_error(), 0.0);
    assert_eq!(summary.successful_metric_summary(), None);
}

#[test]
fn recovery_metric_summary_rejects_invalid_denominators_and_samples() {
    assert_eq!(
        summarize_recovery_metric_replications(0, &[], 0.025, 0.975),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        summarize_recovery_metric_replications(1, &[0.1, 0.2], 0.025, 0.975),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        summarize_recovery_metric_replications(1, &[f64::NAN], 0.025, 0.975),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        summarize_recovery_metric_replications(1, &[0.1], -0.1, 0.975),
        Err(ValidationError::InvalidConfiguration)
    );
    assert_eq!(
        summarize_recovery_metric_replications(1, &[0.1], 0.025, 1.1),
        Err(ValidationError::InvalidConfiguration)
    );
    assert_eq!(
        summarize_recovery_metric_replications(1, &[0.1], 0.9, 0.1),
        Err(ValidationError::InvalidConfiguration)
    );
}

#[test]
fn recovery_metric_summary_handles_all_success() {
    let summary = summarize_recovery_metric_replications(2, &[0.2, 0.4], 0.0, 1.0)
        .expect("all-success recovery summary");

    assert_eq!(summary.attempted_replication_count(), 2);
    assert_eq!(summary.successful_replication_count(), 2);
    assert_eq!(summary.failure_count(), 0);
    assert_eq!(summary.failure_rate(), 0.0);
    assert_eq!(summary.failure_rate_standard_error(), 0.0);
    assert_eq!(
        summary
            .successful_metric_summary()
            .expect("successful metric summary")
            .replication_count,
        2
    );
}
