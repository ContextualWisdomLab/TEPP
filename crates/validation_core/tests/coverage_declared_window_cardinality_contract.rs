use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    CoverageCalibrationReplicationOutcome, ValidationError,
};

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const SCENARIO_ID: &str = "tepp.simulation.rolling_origin_coverage.v1";
const SCENARIO_FINGERPRINT: &str =
    "e5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a";

#[test]
fn persisted_evidence_rejects_a_success_missing_one_declared_rolling_origin_window() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let mut outcomes: Vec<_> = (0..design.attempted_dgp_count())
        .map(CoverageCalibrationReplicationOutcome::numerical_failure)
        .collect();

    // The v1 execution owner declares five rolling-origin windows. Omitting one
    // window must be structural invalidity, not a successful DGP with a different
    // equal-window estimand.
    outcomes[0] = CoverageCalibrationReplicationOutcome::successful(
        0,
        vec![0.95, 0.95, 0.95, 0.95],
    );

    assert_eq!(
        CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
            &design,
            &outcomes,
            SCENARIO_ID,
            SCENARIO_FINGERPRINT,
            SOURCE_HEAD,
        ),
        Err(ValidationError::InvalidInput)
    );
}
