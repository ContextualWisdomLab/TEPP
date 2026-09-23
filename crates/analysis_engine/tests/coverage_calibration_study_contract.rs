use analysis_engine::{
    CoverageCalibrationStudyError, assemble_coverage_calibration_evidence_v1,
    execute_coverage_calibration_shard,
};
use tepp_simulation::CoverageCalibrationSimulationDesign;
use validation_core::CoverageCalibrationReplicationOutcome;

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

fn successful_outcomes() -> Vec<CoverageCalibrationReplicationOutcome> {
    (0..10_000)
        .map(|replication_index| {
            CoverageCalibrationReplicationOutcome::successful(
                replication_index,
                vec![0.95, 0.95, 0.95, 0.95, 0.95],
            )
        })
        .collect()
}

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
fn evidence_assembly_derives_the_versioned_scenario_identity_and_fingerprint() {
    let simulation = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let expected_fingerprint = simulation
        .scenario_fingerprint()
        .expect("versioned scenario fingerprint");
    let record = assemble_coverage_calibration_evidence_v1(&successful_outcomes(), SOURCE_HEAD)
        .expect("owner-bound coverage evidence");

    assert_eq!(record.validation_design_id(), "tepp.coverage.nominal95.v1");
    assert_eq!(record.simulation_scenario_id(), simulation.scenario_id());
    assert_eq!(
        record.simulation_scenario_fingerprint(),
        expected_fingerprint
    );
    assert_eq!(record.source_head(), SOURCE_HEAD);
    assert_eq!(record.attempted_replication_count(), 10_000);
    assert_eq!(record.successful_replication_count(), 10_000);
    assert!(record.supports_calibration_claim());
}

#[test]
fn evidence_assembly_fails_closed_on_noncanonical_source_identity() {
    assert_eq!(
        assemble_coverage_calibration_evidence_v1(&successful_outcomes(), "not-a-git-head"),
        Err(CoverageCalibrationStudyError::InvalidEvidence)
    );
}
