use validation_core::{
    ValidationError, summarize_windowed_coverage_recovery_replications,
};

#[test]
fn windowed_coverage_recovery_keeps_attempted_dgp_denominator() {
    let summary = summarize_windowed_coverage_recovery_replications(
        4,
        &[vec![1.0, 0.8], vec![0.9, 0.9]],
        0.025,
        0.975,
    )
    .expect("coverage recovery summary");

    assert_eq!(summary.attempted_replication_count(), 4);
    assert_eq!(summary.successful_replication_count(), 2);
    assert_eq!(summary.failure_count(), 2);
    assert!((summary.failure_rate() - 0.5).abs() < 1.0e-12);
    assert!((summary.failure_rate_standard_error() - 0.25).abs() < 1.0e-12);
    assert!((summary.successful_metric_mean().expect("coverage mean") - 0.9).abs() < 1.0e-12);
    assert_eq!(
        summary
            .successful_metric_summary()
            .expect("two successful DGPs have estimable Monte Carlo uncertainty")
            .replication_count,
        2
    );
}

#[test]
fn windowed_coverage_recovery_preserves_all_failed_and_single_success_semantics() {
    let all_failed = summarize_windowed_coverage_recovery_replications(
        3,
        &[],
        0.025,
        0.975,
    )
    .expect("all-failed admissible coverage experiment");
    assert_eq!(all_failed.failure_count(), 3);
    assert_eq!(all_failed.failure_rate(), 1.0);
    assert_eq!(all_failed.successful_metric_mean(), None);
    assert_eq!(all_failed.successful_metric_summary(), None);

    let one_success = summarize_windowed_coverage_recovery_replications(
        3,
        &[vec![0.8, 1.0]],
        0.025,
        0.975,
    )
    .expect("single-success admissible coverage experiment");
    assert_eq!(one_success.successful_replication_count(), 1);
    assert!((one_success.successful_metric_mean().expect("coverage mean") - 0.9).abs() < 1.0e-12);
    assert_eq!(one_success.successful_metric_summary(), None);
}

#[test]
fn windowed_coverage_recovery_rejects_structurally_invalid_success_payloads() {
    assert_eq!(
        summarize_windowed_coverage_recovery_replications(0, &[], 0.025, 0.975),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        summarize_windowed_coverage_recovery_replications(
            1,
            &[vec![0.9], vec![0.8]],
            0.025,
            0.975,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        summarize_windowed_coverage_recovery_replications(
            2,
            &[vec![]],
            0.025,
            0.975,
        ),
        Err(ValidationError::InvalidInput)
    );
    for invalid in [f64::NAN, -0.1, 1.1] {
        assert_eq!(
            summarize_windowed_coverage_recovery_replications(
                2,
                &[vec![invalid]],
                0.025,
                0.975,
            ),
            Err(ValidationError::InvalidInput)
        );
    }
    assert_eq!(
        summarize_windowed_coverage_recovery_replications(
            2,
            &[vec![0.9]],
            0.9,
            0.1,
        ),
        Err(ValidationError::InvalidConfiguration)
    );
}
