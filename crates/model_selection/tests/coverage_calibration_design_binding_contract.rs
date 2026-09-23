use std::collections::BTreeSet;

use tepp_simulation::CoverageCalibrationSimulationDesign;
use validation_core::CoverageCalibrationDesign;

const CI_COVERAGE_REPLICATION_SEEDS: [u64; 4] = [101, 211, 307, 401];

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
