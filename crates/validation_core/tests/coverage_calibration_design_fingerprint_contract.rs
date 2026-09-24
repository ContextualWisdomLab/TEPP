use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    CoverageCalibrationReplicationOutcome, coverage_calibration_design_sha256,
};

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const SCENARIO_ID: &str = "tepp.simulation.rolling_origin_coverage.v1";
const SCENARIO_FINGERPRINT: &str =
    "e5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a";
const DESIGN_FINGERPRINT_V1: &str =
    "a3a0b8d65388627d2360b05731c5e410feac343a6068da4f9192a353791bc0c0";

#[test]
fn prospective_design_has_a_pinned_canonical_fingerprint_and_evidence_binding() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
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
        0.025,
        0.975,
        SCENARIO_ID,
        SCENARIO_FINGERPRINT,
        SOURCE_HEAD,
    )
    .expect("calibration evidence");

    assert_eq!(record.schema_version(), 4);
    assert_eq!(record.validation_design_fingerprint(), DESIGN_FINGERPRINT_V1);
    let json = record.to_json().expect("deterministic evidence json");
    assert!(json.contains("\"validation_design_fingerprint\":"));
    assert!(json.contains(DESIGN_FINGERPRINT_V1));
}
