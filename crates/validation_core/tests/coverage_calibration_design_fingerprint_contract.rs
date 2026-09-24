use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    CoverageCalibrationReplicationOutcome, coverage_calibration_design_sha256,
    summarize_replications,
};

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const SCENARIO_ID: &str = "tepp.simulation.rolling_origin_coverage.v1";
const SCENARIO_FINGERPRINT: &str =
    "e5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a";
const ESTIMAND_ID: &str =
    "tepp.coverage.estimand.training_fit_alr_marginal_equal_window.v1";
const ROLLING_ORIGIN_GEOMETRY_ID: &str =
    "tepp.coverage.rolling_origin_geometry.latest_event_availability_adjacent_expanding.v1";
const FAILURE_RATE_MCSE_METHOD_ID: &str =
    "tepp.coverage.failure_rate_mcse.bernoulli_plugin_sqrt_p_one_minus_p_over_n.v1";
const MCSE_METHOD_ID: &str = "tepp.coverage.mcse.sample_sd_n_minus_1_over_sqrt_n.v1";
const PERCENTILE_METHOD_ID: &str = "tepp.coverage.percentile.inclusive_nearest_rank.v1";
const DESIGN_FINGERPRINT_V1: &str =
    "f6fd3855ec7ef413309cbb0b48b3572853d4fa87d4c46e47f265e06074aeffa0";

#[test]
fn prospective_design_has_a_pinned_canonical_fingerprint_and_evidence_binding() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    assert_eq!(design.rolling_origin_geometry_id(), ROLLING_ORIGIN_GEOMETRY_ID);
    assert_eq!(design.first_training_event_index(), 3);
    assert_eq!(design.declared_rolling_origin_window_count(), 5);
    assert_eq!(
        design.failure_rate_monte_carlo_standard_error_method_id(),
        FAILURE_RATE_MCSE_METHOD_ID
    );
    assert_eq!(
        design.coverage_monte_carlo_standard_error_method_id(),
        MCSE_METHOD_ID
    );
    assert_eq!(design.coverage_percentile_method_id(), PERCENTILE_METHOD_ID);

    let fingerprint = coverage_calibration_design_sha256(&design)
        .expect("prospective validation design fingerprint");
    assert_eq!(fingerprint, DESIGN_FINGERPRINT_V1);
    assert_eq!(fingerprint.len(), 64);
    assert!(
        fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );

    let outcomes: Vec<_> = (0..design.attempted_dgp_count())
        .map(CoverageCalibrationReplicationOutcome::numerical_failure)
        .collect();
    let record = CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
        &design,
        &outcomes,
        SCENARIO_ID,
        SCENARIO_FINGERPRINT,
        SOURCE_HEAD,
    )
    .expect("calibration evidence");

    assert_eq!(record.schema_version(), 9);
    assert_eq!(record.validation_estimand_id(), ESTIMAND_ID);
    assert_eq!(record.validation_design_fingerprint(), DESIGN_FINGERPRINT_V1);
    assert_eq!(
        record.failure_rate_monte_carlo_standard_error_method_id(),
        FAILURE_RATE_MCSE_METHOD_ID
    );
    assert_eq!(
        record.coverage_monte_carlo_standard_error_method_id(),
        MCSE_METHOD_ID
    );
    assert_eq!(record.coverage_percentile_method_id(), PERCENTILE_METHOD_ID);
    assert_eq!(record.coverage_percentile_lower_probability(), 0.025);
    assert_eq!(record.coverage_percentile_upper_probability(), 0.975);
    let json = record.to_json().expect("deterministic evidence json");
    assert!(json.contains("\"validation_estimand_id\":"));
    assert!(json.contains(ESTIMAND_ID));
    assert!(json.contains("\"validation_design_fingerprint\":"));
    assert!(json.contains(DESIGN_FINGERPRINT_V1));
    assert!(json.contains("\"failure_rate_monte_carlo_standard_error_method_id\":"));
    assert!(json.contains(FAILURE_RATE_MCSE_METHOD_ID));
    assert!(json.contains("\"coverage_monte_carlo_standard_error_method_id\":"));
    assert!(json.contains(MCSE_METHOD_ID));
    assert!(json.contains("\"coverage_percentile_method_id\":"));
    assert!(json.contains(PERCENTILE_METHOD_ID));
}

#[test]
fn monte_carlo_standard_error_method_identity_matches_owner_arithmetic() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    assert_eq!(
        design.coverage_monte_carlo_standard_error_method_id(),
        MCSE_METHOD_ID
    );

    let summary = summarize_replications(&[1.0, 2.0, 3.0, 4.0], 0.25, 0.75)
        .expect("sample-SD Monte Carlo standard error oracle");
    let expected_standard_deviation = (5.0_f64 / 3.0).sqrt();
    let expected_standard_error = expected_standard_deviation / 2.0;
    assert!((summary.standard_deviation - expected_standard_deviation).abs() < 1.0e-12);
    assert!((summary.standard_error - expected_standard_error).abs() < 1.0e-12);
}

#[test]
fn percentile_method_identity_matches_the_owner_nearest_rank_arithmetic() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    assert_eq!(design.coverage_percentile_method_id(), PERCENTILE_METHOD_ID);

    let summary = summarize_replications(&[1.0, 2.0, 3.0, 4.0], 0.25, 0.75)
        .expect("nearest-rank percentile oracle");
    assert_eq!(summary.percentile_lower, 1.0);
    assert_eq!(summary.percentile_upper, 3.0);
}
