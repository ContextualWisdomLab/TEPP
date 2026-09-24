use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    CoverageCalibrationReplicationOutcome, ValidationError,
};

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const SCENARIO_ID: &str = "tepp.simulation.rolling_origin_coverage.v1";
const SCENARIO_FINGERPRINT: &str =
    "e5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a";

fn persisted_json() -> String {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let windows = vec![0.95; design.declared_rolling_origin_window_count()];
    let outcomes: Vec<_> = (0..10_000)
        .map(|replication_index| {
            CoverageCalibrationReplicationOutcome::successful(
                replication_index,
                windows.clone(),
            )
        })
        .collect();
    CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
        &design,
        &outcomes,
        SCENARIO_ID,
        SCENARIO_FINGERPRINT,
        SOURCE_HEAD,
    )
    .expect("valid calibration evidence")
    .to_json()
    .expect("persisted evidence JSON")
}

#[test]
fn persisted_evidence_recovery_rejects_duplicate_json_members() {
    let json = persisted_json();
    let duplicate_top_level = json.replacen(
        "{\"schema_version\":9,",
        "{\"schema_version\":9,\"schema_version\":9,",
        1,
    );
    assert_eq!(
        CoverageCalibrationEvidenceRecord::from_json(&duplicate_top_level),
        Err(ValidationError::InvalidInput)
    );

    let duplicate_outcome_member = json.replacen(
        "\"replication_index\":0,\"window_coverages\":[0.95,0.95,0.95,0.95,0.95]",
        "\"replication_index\":0,\"replication_index\":0,\"window_coverages\":[0.95,0.95,0.95,0.95,0.95]",
        1,
    );
    assert_eq!(
        CoverageCalibrationEvidenceRecord::from_json(&duplicate_outcome_member),
        Err(ValidationError::InvalidInput)
    );
}
