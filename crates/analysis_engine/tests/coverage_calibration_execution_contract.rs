//! Scientific coverage calibration must execute through one indexed owner path.

use analysis_engine::{
    CoverageCalibrationExecutionError, execute_coverage_calibration_replication,
};
use model_selection::ModelSelectionError;
use tepp_simulation::CoverageCalibrationSimulationDesign;
use validation_core::{CoverageCalibrationDesign, CoverageCalibrationEstimand};

#[test]
fn declared_coverage_executor_is_indexed_and_fail_closed() {
    let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    assert_eq!(
        validation_design.estimand(),
        CoverageCalibrationEstimand::TrainingFitAlrMarginalEqualWindow
    );

    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    assert_eq!(
        execute_coverage_calibration_replication(
            design,
            design.attempted_replication_count(),
        ),
        Err(CoverageCalibrationExecutionError::InvalidSimulationScenario)
    );

    let outcome = execute_coverage_calibration_replication(design, 0)
        .expect("declared replication must be structurally executable");
    assert_eq!(outcome.replication_index(), 0);
    if let Some(window_coverages) = outcome.window_coverages() {
        assert_eq!(window_coverages.len(), 5);
        assert!(
            window_coverages
                .iter()
                .all(|coverage| coverage.is_finite() && (0.0..=1.0).contains(coverage))
        );
    }
}

#[test]
fn execution_errors_keep_stage_specific_diagnostics() {
    for (error, message) in [
        (
            CoverageCalibrationExecutionError::InvalidSimulationScenario,
            "invalid coverage calibration simulation scenario",
        ),
        (
            CoverageCalibrationExecutionError::InvalidTruthManifest,
            "invalid coverage calibration truth manifest",
        ),
        (
            CoverageCalibrationExecutionError::MissingEventAvailability,
            "coverage calibration event has no availability evidence",
        ),
        (
            CoverageCalibrationExecutionError::MissingObservedEventTime,
            "coverage calibration document has no observed event time",
        ),
        (
            CoverageCalibrationExecutionError::InvalidMembershipProjection,
            "invalid coverage calibration membership projection",
        ),
        (
            CoverageCalibrationExecutionError::InvalidRelationProjection,
            "invalid coverage calibration relation projection",
        ),
        (
            CoverageCalibrationExecutionError::InvalidCorpusSnapshot,
            "invalid coverage calibration corpus snapshot",
        ),
        (
            CoverageCalibrationExecutionError::InvalidRollingOriginPartition,
            "invalid coverage calibration rolling-origin partition",
        ),
        (
            CoverageCalibrationExecutionError::InvalidCountMatrix,
            "invalid coverage calibration count matrix",
        ),
        (
            CoverageCalibrationExecutionError::InvalidTrainingInput,
            "invalid coverage calibration topic-training input",
        ),
        (
            CoverageCalibrationExecutionError::ModelSelection(
                ModelSelectionError::RecoveryCandidateInputInvalid,
            ),
            "a declared scientific-recovery candidate is structurally incompatible with the training input",
        ),
        (
            CoverageCalibrationExecutionError::InvalidTopicAlignment,
            "invalid coverage calibration topic alignment",
        ),
        (
            CoverageCalibrationExecutionError::InvalidPosteriorGeometry,
            "invalid coverage calibration posterior geometry",
        ),
        (
            CoverageCalibrationExecutionError::MissingTruthState,
            "coverage calibration fitted document has no truth state",
        ),
        (
            CoverageCalibrationExecutionError::InvalidCoverage,
            "invalid coverage calibration interval coverage",
        ),
        (
            CoverageCalibrationExecutionError::ArithmeticOverflow,
            "coverage calibration identity arithmetic overflow",
        ),
    ] {
        assert_eq!(error.to_string(), message);
    }
}
