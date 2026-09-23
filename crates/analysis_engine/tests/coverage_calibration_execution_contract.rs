//! Scientific coverage calibration must execute through one indexed owner path.

use analysis_engine::{
    CoverageCalibrationExecutionError, execute_coverage_calibration_replication,
};
use tepp_simulation::CoverageCalibrationSimulationDesign;

#[test]
fn declared_coverage_executor_is_indexed_and_fail_closed() {
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
        assert!(window_coverages.iter().all(|coverage| {
            coverage.is_finite() && (0.0..=1.0).contains(coverage)
        }));
    }
}
