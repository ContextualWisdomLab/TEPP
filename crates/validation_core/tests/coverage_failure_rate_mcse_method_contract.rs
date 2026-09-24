use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    CoverageCalibrationReplicationOutcome, summarize_recovery_metric_replications,
};

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const SCENARIO_ID: &str = "tepp.simulation.rolling_origin_coverage.v1";
const SCENARIO_FINGERPRINT: &str =
    "e5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a";
const FAILURE_RATE_MCSE_METHOD_ID: &str =
    "tepp.coverage.failure_rate_mcse.bernoulli_plugin_sqrt_p_one_minus_p_over_n.v1";

#[test]
fn failure_rate_mcse_method_identity_matches_owner_arithmetic() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    assert_eq!(
        design.failure_rate_monte_carlo_standard_error_method_id(),
        FAILURE_RATE_MCSE_METHOD_ID
    );

    let summary = summarize_recovery_metric_replications(
        4,
        &[0.90, 0.95, 1.00],
        design.coverage_percentile_lower_probability(),
        design.coverage_percentile_upper_probability(),
    )
    .expect("denominator-preserving recovery summary");
    let expected = (0.25_f64 * 0.75 / 4.0).sqrt();
    assert!((summary.failure_rate_standard_error() - expected).abs() < 1.0e-12);
}

#[test]
fn durable_evidence_names_the_failure_rate_mcse_method() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
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
    .expect("coverage evidence");

    assert_eq!(record.schema_version(), 9);
    assert_eq!(
        record.failure_rate_monte_carlo_standard_error_method_id(),
        FAILURE_RATE_MCSE_METHOD_ID
    );
    let json = record.to_json().expect("deterministic evidence json");
    assert!(json.contains("\"failure_rate_monte_carlo_standard_error_method_id\":"));
    assert!(json.contains(FAILURE_RATE_MCSE_METHOD_ID));
}
