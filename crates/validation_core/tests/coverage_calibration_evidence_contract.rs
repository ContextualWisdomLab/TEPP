use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    CoverageCalibrationReplicationOutcome, ValidationError,
};

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const SCENARIO_ID: &str = "tepp.simulation.rolling_origin_coverage.v1";
const SCENARIO_FINGERPRINT: &str =
    "e5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a";
const LOWER_PERCENTILE: f64 = 0.025;
const UPPER_PERCENTILE: f64 = 0.975;

fn outcomes(successful: usize) -> Vec<CoverageCalibrationReplicationOutcome> {
    (0..10_000)
        .map(|replication_index| {
            if replication_index < successful {
                CoverageCalibrationReplicationOutcome::successful(replication_index, vec![0.95])
            } else {
                CoverageCalibrationReplicationOutcome::numerical_failure(replication_index)
            }
        })
        .collect()
}

fn outcomes_with_failures(failure_indices: &[usize]) -> Vec<CoverageCalibrationReplicationOutcome> {
    (0..10_000)
        .map(|replication_index| {
            if failure_indices.contains(&replication_index) {
                CoverageCalibrationReplicationOutcome::numerical_failure(replication_index)
            } else {
                CoverageCalibrationReplicationOutcome::successful(replication_index, vec![0.95])
            }
        })
        .collect()
}

fn evidence(successful: usize) -> Result<CoverageCalibrationEvidenceRecord, ValidationError> {
    CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
        &CoverageCalibrationDesign::tepp_nominal_95_v1(),
        &outcomes(successful),
        LOWER_PERCENTILE,
        UPPER_PERCENTILE,
        SCENARIO_ID,
        SCENARIO_FINGERPRINT,
        SOURCE_HEAD,
    )
}

fn evidence_from_outcomes(
    outcomes: &[CoverageCalibrationReplicationOutcome],
) -> Result<CoverageCalibrationEvidenceRecord, ValidationError> {
    CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
        &CoverageCalibrationDesign::tepp_nominal_95_v1(),
        outcomes,
        LOWER_PERCENTILE,
        UPPER_PERCENTILE,
        SCENARIO_ID,
        SCENARIO_FINGERPRINT,
        SOURCE_HEAD,
    )
}

#[test]
fn calibration_evidence_binds_design_scenario_source_denominator_and_uncertainty() {
    let record = evidence(9_998).expect("calibration evidence");

    assert_eq!(record.schema_version(), 1);
    assert_eq!(record.validation_design_id(), "tepp.coverage.nominal95.v1");
    assert_eq!(record.simulation_scenario_id(), SCENARIO_ID);
    assert_eq!(record.simulation_scenario_fingerprint(), SCENARIO_FINGERPRINT);
    assert_eq!(record.source_head(), SOURCE_HEAD);
    assert_eq!(record.attempted_replication_count(), 10_000);
    assert_eq!(record.successful_replication_count(), 9_998);
    assert_eq!(record.failure_count(), 2);
    assert!((record.failure_rate() - 0.0002).abs() < f64::EPSILON);
    assert!(record.failure_rate_standard_error().is_finite());
    assert!(record.failure_rate_standard_error() > 0.0);
    assert_eq!(record.coverage_mean(), Some(0.95));
    assert_eq!(record.coverage_standard_deviation(), Some(0.0));
    assert_eq!(record.coverage_monte_carlo_standard_error(), Some(0.0));
    assert_eq!(record.coverage_percentile_lower_probability(), LOWER_PERCENTILE);
    assert_eq!(record.coverage_percentile_upper_probability(), UPPER_PERCENTILE);
    assert_eq!(record.coverage_percentile_lower(), Some(0.95));
    assert_eq!(record.coverage_percentile_upper(), Some(0.95));
    assert!(record.coverage_within_practical_band());
    assert!(record.monte_carlo_precision_sufficient());
    assert!(record.supports_calibration_claim());

    let json = record.to_json().expect("deterministic evidence json");
    assert!(json.contains("\"schema_version\":1"));
    assert!(json.contains("\"attempted_replication_count\":10000"));
    assert!(json.contains("\"successful_replication_count\":9998"));
    assert!(json.contains("\"failure_count\":2"));
    assert!(json.contains("\"failure_rate_standard_error\":"));
    assert!(json.contains("\"coverage_percentile_lower_probability\":0.025"));
    assert!(json.contains("\"coverage_percentile_upper_probability\":0.975"));
    assert!(json.contains("\"coverage_percentile_lower\":0.95"));
    assert!(json.contains("\"coverage_percentile_upper\":0.95"));
    assert!(json.contains(SCENARIO_FINGERPRINT));
    assert!(json.contains(SOURCE_HEAD));
    assert_eq!(record.to_json().expect("repeat json"), json);
}

#[test]
fn persisted_evidence_distinguishes_exact_indexed_failure_identity_pattern() {
    let early_failures = evidence_from_outcomes(&outcomes_with_failures(&[0, 1]))
        .expect("early-failure evidence");
    let late_failures = evidence_from_outcomes(&outcomes_with_failures(&[9_998, 9_999]))
        .expect("late-failure evidence");

    assert_eq!(early_failures.attempted_replication_count(), 10_000);
    assert_eq!(late_failures.attempted_replication_count(), 10_000);
    assert_eq!(early_failures.successful_replication_count(), 9_998);
    assert_eq!(late_failures.successful_replication_count(), 9_998);
    assert_eq!(early_failures.failure_count(), 2);
    assert_eq!(late_failures.failure_count(), 2);
    assert_eq!(early_failures.coverage_mean(), Some(0.95));
    assert_eq!(late_failures.coverage_mean(), Some(0.95));
    assert_ne!(
        early_failures.to_json().expect("early json"),
        late_failures.to_json().expect("late json"),
        "persisted evidence must retain which declared DGP identities failed"
    );
}

#[test]
fn singleton_success_keeps_point_evidence_without_fabricating_dispersion() {
    let record = evidence(1).expect("singleton evidence remains reportable");

    assert_eq!(record.coverage_mean(), Some(0.95));
    assert_eq!(record.coverage_standard_deviation(), None);
    assert_eq!(record.coverage_monte_carlo_standard_error(), None);
    assert_eq!(record.coverage_percentile_lower(), None);
    assert_eq!(record.coverage_percentile_upper(), None);
    assert!(!record.monte_carlo_precision_sufficient());
    assert!(!record.supports_calibration_claim());
}

#[test]
fn calibration_evidence_fails_closed_on_identity_digest_head_or_percentile_drift() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let valid_outcomes = outcomes(2);

    for (scenario_id, fingerprint, source_head) in [
        ("", SCENARIO_FINGERPRINT, SOURCE_HEAD),
        ("tepp.\u{1}scenario", SCENARIO_FINGERPRINT, SOURCE_HEAD),
        (SCENARIO_ID, "abc", SOURCE_HEAD),
        (
            SCENARIO_ID,
            "E5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a",
            SOURCE_HEAD,
        ),
        (
            SCENARIO_ID,
            SCENARIO_FINGERPRINT,
            "0123456789ABCDEF0123456789abcdef01234567",
        ),
        (SCENARIO_ID, SCENARIO_FINGERPRINT, "not-a-git-head"),
    ] {
        assert_eq!(
            CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
                &design,
                &valid_outcomes,
                LOWER_PERCENTILE,
                UPPER_PERCENTILE,
                scenario_id,
                fingerprint,
                source_head,
            ),
            Err(ValidationError::InvalidInput)
        );
    }

    assert_eq!(
        CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
            &design,
            &valid_outcomes,
            0.9,
            0.1,
            SCENARIO_ID,
            SCENARIO_FINGERPRINT,
            SOURCE_HEAD,
        ),
        Err(ValidationError::InvalidConfiguration)
    );
}
