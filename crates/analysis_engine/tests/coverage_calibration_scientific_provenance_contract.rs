use analysis_engine::{
    CoverageCalibrationScientificShardRecord, CoverageCalibrationStudyError,
    assemble_coverage_calibration_scientific_evidence_v1,
    execute_coverage_calibration_scientific_shard_record,
};
use model_selection::CoverageCalibrationFitDesign;
use tepp_simulation::CoverageCalibrationSimulationDesign;

const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn scientific_shard_binds_the_owner_issued_numerical_fit_schedule() {
    let simulation_design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let fit_design = CoverageCalibrationFitDesign::truth_k_v1();
    let record = execute_coverage_calibration_scientific_shard_record(
        simulation_design,
        0,
        1,
        SOURCE_HEAD,
    )
    .expect("first declared scientific shard");

    assert_eq!(record.schema_version(), 1);
    assert_eq!(record.fit_design_id(), fit_design.design_id());
    assert_eq!(
        record.fit_schedule_sha256(),
        "9180e8d84492560225ed689be198b75f6f3398071ce53f884aef7b3bce422dc3"
    );
    assert_eq!(record.shard_record().source_head(), SOURCE_HEAD);
    assert_eq!(record.shard_record().start_replication_index(), 0);
    assert_eq!(record.shard_record().end_replication_index_exclusive(), 1);

    let json = record.to_json().expect("scientific shard json");
    let digest = record.sha256().expect("scientific shard digest");
    let recovered = CoverageCalibrationScientificShardRecord::from_json_with_sha256(
        &json,
        &digest,
    )
    .expect("strict scientific shard recovery");
    assert_eq!(recovered, record);

    let mut drifted: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    drifted["fit_schedule_sha256"] = serde_json::json!(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert_eq!(
        CoverageCalibrationScientificShardRecord::from_json_with_sha256(
            &drifted.to_string(),
            &digest,
        ),
        Err(CoverageCalibrationStudyError::InvalidShardProvenance)
    );
}

#[test]
fn scientific_evidence_refuses_an_incomplete_fit_bound_shard_set() {
    let simulation_design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let shard = execute_coverage_calibration_scientific_shard_record(
        simulation_design,
        0,
        1,
        SOURCE_HEAD,
    )
    .expect("first declared scientific shard");

    assert_eq!(
        assemble_coverage_calibration_scientific_evidence_v1(&[shard]),
        Err(CoverageCalibrationStudyError::IncompleteShardSet)
    );
}
