use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord, ValidationError,
    summarize_windowed_coverage_recovery_replications,
};

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const SCENARIO_ID: &str = "tepp.simulation.rolling_origin_coverage.v1";
const SCENARIO_FINGERPRINT: &str =
    "e5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a";

fn summary(attempted: usize, successful: usize) -> validation_core::MonteCarloRecoveryMetricSummary {
    let successful_coverages = vec![vec![0.95]; successful];
    summarize_windowed_coverage_recovery_replications(
        attempted,
        &successful_coverages,
        0.025,
        0.975,
    )
    .expect("denominator-preserving coverage summary")
}

#[test]
fn calibration_evidence_binds_design_scenario_source_and_failure_uncertainty() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let summary = summary(10_000, 9_998);
    let record = CoverageCalibrationEvidenceRecord::from_summary(
        &design,
        &summary,
        SCENARIO_ID,
        SCENARIO_FINGERPRINT,
        SOURCE_HEAD,
    )
    .expect("calibration evidence");

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
    assert!(json.contains("\"coverage_percentile_lower\":0.95"));
    assert!(json.contains("\"coverage_percentile_upper\":0.95"));
    assert!(json.contains(SCENARIO_FINGERPRINT));
    assert!(json.contains(SOURCE_HEAD));
    assert_eq!(record.to_json().expect("repeat json"), json);
}

#[test]
fn singleton_success_keeps_point_evidence_without_fabricating_dispersion() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let singleton = summary(10_000, 1);
    let record = CoverageCalibrationEvidenceRecord::from_summary(
        &design,
        &singleton,
        SCENARIO_ID,
        SCENARIO_FINGERPRINT,
        SOURCE_HEAD,
    )
    .expect("singleton evidence remains reportable");

    assert_eq!(record.coverage_mean(), Some(0.95));
    assert_eq!(record.coverage_standard_deviation(), None);
    assert_eq!(record.coverage_monte_carlo_standard_error(), None);
    assert_eq!(record.coverage_percentile_lower(), None);
    assert_eq!(record.coverage_percentile_upper(), None);
    assert!(!record.monte_carlo_precision_sufficient());
    assert!(!record.supports_calibration_claim());
}

#[test]
fn calibration_evidence_fails_closed_on_identity_digest_head_or_design_drift() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let valid_summary = summary(10_000, 2);

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
            CoverageCalibrationEvidenceRecord::from_summary(
                &design,
                &valid_summary,
                scenario_id,
                fingerprint,
                source_head,
            ),
            Err(ValidationError::InvalidInput)
        );
    }

    let wrong_attempt_count = summary(9_999, 2);
    assert_eq!(
        CoverageCalibrationEvidenceRecord::from_summary(
            &design,
            &wrong_attempt_count,
            SCENARIO_ID,
            SCENARIO_FINGERPRINT,
            SOURCE_HEAD,
        ),
        Err(ValidationError::InvalidInput)
    );
}
