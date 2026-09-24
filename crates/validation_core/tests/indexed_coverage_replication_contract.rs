use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationReplicationOutcome, ValidationError,
    summarize_indexed_windowed_coverage_recovery_replications,
};

fn complete_windows(value: f64) -> Vec<f64> {
    vec![
        value;
        CoverageCalibrationDesign::tepp_nominal_95_v1().declared_rolling_origin_window_count()
    ]
}

#[test]
fn indexed_coverage_aggregation_preserves_exact_denominator_and_canonical_order() {
    let outcomes = [
        CoverageCalibrationReplicationOutcome::successful(2, complete_windows(0.85)),
        CoverageCalibrationReplicationOutcome::numerical_failure(0),
        CoverageCalibrationReplicationOutcome::successful(1, complete_windows(0.95)),
    ];

    assert_eq!(outcomes[0].replication_index(), 2);
    assert_eq!(outcomes[0].window_coverages(), Some(&[0.85; 5][..]));
    assert_eq!(outcomes[1].replication_index(), 0);
    assert_eq!(outcomes[1].window_coverages(), None);

    let summary = summarize_indexed_windowed_coverage_recovery_replications(
        3,
        &outcomes,
        0.025,
        0.975,
    )
    .expect("exact indexed denominator");

    assert_eq!(summary.attempted_replication_count(), 3);
    assert_eq!(summary.successful_replication_count(), 2);
    assert_eq!(summary.failure_count(), 1);
    assert!((summary.failure_rate() - (1.0 / 3.0)).abs() < f64::EPSILON);
    assert!((summary.successful_metric_mean().expect("mean") - 0.9).abs() < 1e-12);
}

#[test]
fn indexed_coverage_aggregation_rejects_duplicate_missing_and_out_of_range_identity() {
    for outcomes in [
        vec![
            CoverageCalibrationReplicationOutcome::numerical_failure(0),
            CoverageCalibrationReplicationOutcome::numerical_failure(0),
            CoverageCalibrationReplicationOutcome::numerical_failure(2),
        ],
        vec![
            CoverageCalibrationReplicationOutcome::numerical_failure(0),
            CoverageCalibrationReplicationOutcome::numerical_failure(2),
        ],
        vec![
            CoverageCalibrationReplicationOutcome::numerical_failure(0),
            CoverageCalibrationReplicationOutcome::numerical_failure(1),
            CoverageCalibrationReplicationOutcome::numerical_failure(3),
        ],
    ] {
        assert_eq!(
            summarize_indexed_windowed_coverage_recovery_replications(
                3,
                &outcomes,
                0.025,
                0.975,
            ),
            Err(ValidationError::InvalidInput)
        );
    }

    assert_eq!(
        summarize_indexed_windowed_coverage_recovery_replications(0, &[], 0.025, 0.975),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn indexed_coverage_enforces_declared_window_cardinality_and_delegates_value_validation() {
    let empty_success = [CoverageCalibrationReplicationOutcome::successful(0, vec![])];
    assert_eq!(
        summarize_indexed_windowed_coverage_recovery_replications(
            1,
            &empty_success,
            0.025,
            0.975,
        ),
        Err(ValidationError::InvalidInput)
    );

    let shortened_success = [CoverageCalibrationReplicationOutcome::successful(
        0,
        vec![0.95; 4],
    )];
    assert_eq!(
        summarize_indexed_windowed_coverage_recovery_replications(
            1,
            &shortened_success,
            0.025,
            0.975,
        ),
        Err(ValidationError::InvalidInput)
    );

    let mut invalid_windows = complete_windows(0.95);
    invalid_windows[2] = 1.1;
    let invalid_coverage = [CoverageCalibrationReplicationOutcome::successful(
        0,
        invalid_windows,
    )];
    assert_eq!(
        summarize_indexed_windowed_coverage_recovery_replications(
            1,
            &invalid_coverage,
            0.025,
            0.975,
        ),
        Err(ValidationError::InvalidInput)
    );

    let valid = [CoverageCalibrationReplicationOutcome::successful(
        0,
        complete_windows(0.95),
    )];
    assert_eq!(
        summarize_indexed_windowed_coverage_recovery_replications(1, &valid, 0.9, 0.1),
        Err(ValidationError::InvalidConfiguration)
    );
}
