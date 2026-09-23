use std::collections::BTreeSet;

use tepp_simulation::CoverageCalibrationSimulationDesign;
use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    summarize_windowed_coverage_recovery_replications,
};

const CI_COVERAGE_REPLICATION_SEEDS: [u64; 4] = [101, 211, 307, 401];
const TEST_SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn prospective_coverage_scenario_binds_to_validation_design_without_copied_counts() {
    let scenario = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let criterion = CoverageCalibrationDesign::tepp_nominal_95_v1();

    assert_ne!(scenario.scenario_id(), criterion.design_id());
    assert_eq!(
        scenario.attempted_replication_count(),
        criterion.attempted_dgp_count()
    );
}

#[test]
fn prospective_coverage_seed_schedule_is_unique_and_disjoint_from_ci_fixture() {
    let scenario = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let acceptance_seeds: BTreeSet<_> = (0..scenario.attempted_replication_count())
        .map(|index| {
            scenario
                .seed_for_replication(index)
                .expect("declared acceptance replication seed")
        })
        .collect();

    assert_eq!(
        acceptance_seeds.len(),
        scenario.attempted_replication_count()
    );
    for ci_seed in CI_COVERAGE_REPLICATION_SEEDS {
        assert!(
            !acceptance_seeds.contains(&ci_seed),
            "prospective acceptance seed schedule overlaps CI seed {ci_seed}"
        );
    }
}

#[test]
fn persisted_calibration_evidence_uses_owner_scenario_identity_and_fingerprint() {
    let scenario = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let criterion = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let successful_coverages = [vec![0.95], vec![0.95]];
    let summary = summarize_windowed_coverage_recovery_replications(
        criterion.attempted_dgp_count(),
        &successful_coverages,
        0.025,
        0.975,
    )
    .expect("denominator-preserving composition fixture");
    let scenario_fingerprint = scenario
        .scenario_fingerprint()
        .expect("owner-issued scenario fingerprint");

    let evidence = CoverageCalibrationEvidenceRecord::from_summary(
        &criterion,
        &summary,
        scenario.scenario_id(),
        &scenario_fingerprint,
        TEST_SOURCE_HEAD,
    )
    .expect("cross-owner calibration evidence");

    assert_eq!(evidence.validation_design_id(), criterion.design_id());
    assert_eq!(evidence.simulation_scenario_id(), scenario.scenario_id());
    assert_eq!(
        evidence.simulation_scenario_fingerprint(),
        scenario_fingerprint
    );
    assert_eq!(
        evidence.attempted_replication_count(),
        scenario.attempted_replication_count()
    );
}
