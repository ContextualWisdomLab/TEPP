use analysis_engine::{
    CoverageCalibrationShardRecord, CoverageCalibrationStudyError,
    execute_coverage_calibration_shard_record,
};
use tepp_simulation::CoverageCalibrationSimulationDesign;

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn persisted_shard_rehydration_verifies_the_expected_owner_digest() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");
    let json = record.to_json().expect("owner shard json");
    let digest = record.sha256().expect("owner shard digest");

    let recovered = CoverageCalibrationShardRecord::from_json_with_sha256(&json, &digest)
        .expect("matching owner digest must rehydrate the shard");
    assert_eq!(recovered, record);

    let mut wrong_digest = digest.clone();
    wrong_digest.replace_range(
        0..1,
        if &digest[0..1] == "0" { "1" } else { "0" },
    );
    assert_eq!(
        CoverageCalibrationShardRecord::from_json_with_sha256(&json, &wrong_digest),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );
}

#[test]
fn digest_verified_rehydration_rejects_admissible_json_mutation() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");
    let json = record.to_json().expect("owner shard json");
    let digest = record.sha256().expect("owner shard digest");
    let mut mutated: serde_json::Value = serde_json::from_str(&json).expect("valid owner JSON");
    mutated["source_head"] =
        serde_json::json!("1123456789abcdef0123456789abcdef01234567");
    let mutated_json = mutated.to_string();

    assert_eq!(
        CoverageCalibrationShardRecord::from_json_with_sha256(&mutated_json, &digest),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );
}

#[test]
fn digest_verified_rehydration_rejects_noncanonical_expected_digest() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");
    let json = record.to_json().expect("owner shard json");

    assert_eq!(
        CoverageCalibrationShardRecord::from_json_with_sha256(&json, "abc123"),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );
}
