use analysis_engine::{
    CoverageCalibrationStudyError, execute_coverage_calibration_shard,
    execute_coverage_calibration_shard_record,
};
use tepp_simulation::CoverageCalibrationSimulationDesign;

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn shard_execution_uses_declared_replication_ordinals_and_rejects_invalid_ranges() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();

    assert_eq!(
        execute_coverage_calibration_shard(design, 0, 0),
        Err(CoverageCalibrationStudyError::InvalidShardRange)
    );
    assert_eq!(
        execute_coverage_calibration_shard(
            design,
            design.attempted_replication_count(),
            design.attempted_replication_count() + 1,
        ),
        Err(CoverageCalibrationStudyError::InvalidShardRange)
    );

    let outcomes = execute_coverage_calibration_shard(design, 0, 1)
        .expect("first declared replication must be structurally executable");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].replication_index(), 0);
}

#[test]
fn shard_record_derives_the_versioned_scenario_identity_and_fingerprint() {
    let simulation = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let expected_fingerprint = simulation
        .scenario_fingerprint()
        .expect("versioned scenario fingerprint");
    let record = execute_coverage_calibration_shard_record(simulation, 0, 1, SOURCE_HEAD)
        .expect("owner-bound shard record");

    assert_eq!(record.simulation_scenario_id(), simulation.scenario_id());
    assert_eq!(
        record.simulation_scenario_fingerprint(),
        expected_fingerprint
    );
    assert_eq!(record.source_head(), SOURCE_HEAD);
    assert_eq!(record.start_replication_index(), 0);
    assert_eq!(record.end_replication_index_exclusive(), 1);
    assert_eq!(record.outcomes()[0].replication_index(), 0);
}

#[test]
fn shard_record_fails_closed_on_noncanonical_source_identity() {
    let simulation = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    assert_eq!(
        execute_coverage_calibration_shard_record(simulation, 0, 1, "not-a-git-head"),
        Err(CoverageCalibrationStudyError::InvalidSourceIdentity)
    );
}
