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
const PERCENTILE_METHOD_ID: &str = "tepp.coverage.percentile.inclusive_nearest_rank.v1";
const DESIGN_FINGERPRINT_V1: &str =
    "b4733bdaea49439e1d45749227d5e6131ebd14a0fb56df3dc0878e4fc3bf9954";

#[test]
fn prospective_design_has_a_pinned_canonical_fingerprint_and_evidence_binding() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
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

    assert_eq!(record.schema_version(), 7);
    assert_eq!(record.validation_estimand_id(), ESTIMAND_ID);
    assert_eq!(record.validation_design_fingerprint(), DESIGN_FINGERPRINT_V1);
    assert_eq!(record.coverage_percentile_method_id(), PERCENTILE_METHOD_ID);
    assert_eq!(record.coverage_percentile_lower_probability(), 0.025);
    assert_eq!(record.coverage_percentile_upper_probability(), 0.975);
    let json = record.to_json().expect("deterministic evidence json");
    assert!(json.contains("\"validation_estimand_id\":"));
    assert!(json.contains(ESTIMAND_ID));
    assert!(json.contains("\"validation_design_fingerprint\":"));
    assert!(json.contains(DESIGN_FINGERPRINT_V1));
    assert!(json.contains("\"coverage_percentile_method_id\":"));
    assert!(json.contains(PERCENTILE_METHOD_ID));
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
