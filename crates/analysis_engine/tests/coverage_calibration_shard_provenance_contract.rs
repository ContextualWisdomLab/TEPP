use analysis_engine::{
    CoverageCalibrationShardRecord, CoverageCalibrationStudyError,
    assemble_coverage_calibration_evidence_v1, execute_coverage_calibration_shard_record,
};
use tepp_simulation::CoverageCalibrationSimulationDesign;
use validation_core::{CoverageCalibrationDesign, coverage_calibration_design_sha256};

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn shard_record_binds_declared_validation_design_range_scenario_source_and_digest() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");

    assert_eq!(record.schema_version(), 2);
    assert_eq!(record.start_replication_index(), 0);
    assert_eq!(record.end_replication_index_exclusive(), 1);
    assert_eq!(record.source_head(), SOURCE_HEAD);
    assert_eq!(record.validation_design_id(), validation_design.design_id());
    assert_eq!(
        record.validation_design_fingerprint(),
        coverage_calibration_design_sha256(&validation_design)
            .expect("versioned validation design fingerprint")
    );
    assert_eq!(record.simulation_scenario_id(), design.scenario_id());
    assert_eq!(
        record.simulation_scenario_fingerprint(),
        design
            .scenario_fingerprint()
            .expect("versioned scenario fingerprint")
    );
    assert_eq!(record.outcomes().len(), 1);
    assert_eq!(record.outcomes()[0].replication_index(), 0);
    assert_eq!(record.sha256().expect("shard digest").len(), 64);
    assert!(record.to_json().expect("shard json").contains(SOURCE_HEAD));
}

#[test]
fn persisted_shard_round_trip_rehydrates_the_same_provenance_digest() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");
    let json = record.to_json().expect("owner shard json");

    let recovered = CoverageCalibrationShardRecord::from_json(&json)
        .expect("owner JSON must rehydrate the typed shard record");

    assert_eq!(recovered, record);
    assert_eq!(
        recovered.sha256().expect("recovered shard digest"),
        record.sha256().expect("original shard digest")
    );
}

#[test]
fn persisted_shard_rehydration_rejects_schema_source_fingerprint_range_and_identity_drift() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");
    let json = record.to_json().expect("owner shard json");
    let baseline: serde_json::Value = serde_json::from_str(&json).expect("valid owner JSON");

    let mut wrong_schema = baseline.clone();
    wrong_schema["schema_version"] = serde_json::json!(3);
    assert_eq!(
        CoverageCalibrationShardRecord::from_json(&wrong_schema.to_string()),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );

    let mut wrong_source = baseline.clone();
    wrong_source["source_head"] = serde_json::json!("ABCDEF0123456789abcdef0123456789abcdef01");
    assert_eq!(
        CoverageCalibrationShardRecord::from_json(&wrong_source.to_string()),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );

    let mut wrong_fingerprint = baseline.clone();
    wrong_fingerprint["simulation_scenario_fingerprint"] = serde_json::json!("abc123");
    assert_eq!(
        CoverageCalibrationShardRecord::from_json(&wrong_fingerprint.to_string()),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );

    let mut wrong_validation_fingerprint = baseline.clone();
    wrong_validation_fingerprint["validation_design_fingerprint"] = serde_json::json!("abc123");
    assert_eq!(
        CoverageCalibrationShardRecord::from_json(&wrong_validation_fingerprint.to_string()),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );

    let mut empty_range = baseline.clone();
    empty_range["end_replication_index_exclusive"] = serde_json::json!(0);
    assert_eq!(
        CoverageCalibrationShardRecord::from_json(&empty_range.to_string()),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );

    let mut wrong_identity = baseline;
    wrong_identity["outcomes"][0]["replication_index"] = serde_json::json!(1);
    assert_eq!(
        CoverageCalibrationShardRecord::from_json(&wrong_identity.to_string()),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );
}

#[test]
fn final_assembly_rejects_post_execution_validation_design_substitution() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");
    let json = record.to_json().expect("owner shard json");
    let mut drifted: serde_json::Value = serde_json::from_str(&json).expect("valid owner JSON");
    drifted["validation_design_id"] = serde_json::json!("tepp.coverage.posthoc.v999");
    drifted["validation_design_fingerprint"] =
        serde_json::json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let drifted_record = CoverageCalibrationShardRecord::from_json(&drifted.to_string())
        .expect("syntactically canonical historical design provenance should rehydrate");

    assert_eq!(
        assemble_coverage_calibration_evidence_v1(&[drifted_record]),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );
}

#[test]
fn shard_execution_rejects_noncanonical_source_identity_before_persistence() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    assert_eq!(
        execute_coverage_calibration_shard_record(design, 0, 1, "not-a-git-head"),
        Err(CoverageCalibrationStudyError::InvalidSourceIdentity)
    );
}

#[test]
fn final_evidence_consumes_provenance_bound_shards() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");

    assert_eq!(
        assemble_coverage_calibration_evidence_v1(&[record]),
        Err(CoverageCalibrationStudyError::IncompleteShardSet)
    );
}
