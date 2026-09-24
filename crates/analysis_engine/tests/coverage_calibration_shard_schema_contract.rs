use analysis_engine::{
    CoverageCalibrationShardRecord, CoverageCalibrationStudyError,
    execute_coverage_calibration_shard_record,
};
use tepp_simulation::CoverageCalibrationSimulationDesign;

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn persisted_shard_rehydration_rejects_undeclared_schema_members() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");
    let json = record.to_json().expect("owner shard json");
    let baseline: serde_json::Value = serde_json::from_str(&json).expect("valid owner JSON");

    let mut extra_top_level = baseline.clone();
    extra_top_level["source_sha"] = serde_json::json!(SOURCE_HEAD);
    assert_eq!(
        CoverageCalibrationShardRecord::from_json(&extra_top_level.to_string()),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );

    let mut extra_outcome_member = baseline;
    extra_outcome_member["outcomes"][0]["coverage_mean"] = serde_json::json!(0.95);
    assert_eq!(
        CoverageCalibrationShardRecord::from_json(&extra_outcome_member.to_string()),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );
}
