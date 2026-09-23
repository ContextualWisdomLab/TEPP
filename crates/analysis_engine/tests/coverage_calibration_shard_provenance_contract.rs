use analysis_engine::{
    CoverageCalibrationStudyError, assemble_coverage_calibration_evidence_v1,
    execute_coverage_calibration_shard_record,
};
use tepp_simulation::CoverageCalibrationSimulationDesign;

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn shard_record_binds_declared_range_scenario_source_and_digest() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let record = execute_coverage_calibration_shard_record(design, 0, 1, SOURCE_HEAD)
        .expect("first declared shard must be structurally executable");

    assert_eq!(record.schema_version(), 1);
    assert_eq!(record.start_replication_index(), 0);
    assert_eq!(record.end_replication_index_exclusive(), 1);
    assert_eq!(record.source_head(), SOURCE_HEAD);
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
