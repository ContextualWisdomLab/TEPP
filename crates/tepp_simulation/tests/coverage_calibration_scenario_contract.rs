use std::collections::BTreeSet;

use tepp_simulation::{CoverageCalibrationSimulationDesign, SimulationError, TopicDgpConfig};

#[test]
fn coverage_calibration_v1_owns_exact_seed_schedule_and_realistic_dgp() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();

    assert_eq!(
        design.scenario_id(),
        "tepp.simulation.rolling_origin_coverage.v1"
    );
    assert_eq!(design.attempted_replication_count(), 10_000);

    let first_seed = design.seed_for_replication(0).expect("first seed");
    let last_seed = design
        .seed_for_replication(9_999)
        .expect("last declared seed");
    assert_eq!(first_seed, 0x4341_4c49_4252_0000);
    assert_eq!(last_seed, first_seed + 9_999);
    assert_eq!(
        design.seed_for_replication(10_000),
        Err(SimulationError::InvalidConfiguration)
    );

    let seeds: BTreeSet<_> = (0..design.attempted_replication_count())
        .map(|index| design.seed_for_replication(index).expect("declared seed"))
        .collect();
    assert_eq!(seeds.len(), design.attempted_replication_count());

    let config = design
        .config_for_replication(0)
        .expect("owner-issued coverage calibration config");
    assert_eq!(config.seed(), first_seed);
    assert_eq!(config.event_count(), 9);
    assert_eq!(config.documents_per_event(), 2);
    assert_eq!(config.membership_targets(), 3);
    assert_eq!(config.max_report_delay_hours(), 12);
    assert_eq!(config.max_availability_delay_hours(), 6);
    assert_eq!(config.missingness_rate_bps(), 0);
    assert_eq!(config.relation_false_negative_bps(), 0);
    assert_eq!(config.relation_false_positive_bps(), 500);
    assert_eq!(config.revision_rate_bps(), 3_000);
    assert_eq!(config.translation_rate_bps(), 3_000);
    assert_eq!(config.template_copy_rate_bps(), 3_000);
    assert_eq!(config.topic_dgp(), TopicDgpConfig::ci_default());
}

#[test]
fn coverage_calibration_v1_reuses_owner_dgp_shape_only_for_disjoint_regression_seeds() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let first_seed = design.seed_for_replication(0).expect("first seed");
    let last_seed = design
        .seed_for_replication(design.attempted_replication_count() - 1)
        .expect("last declared seed");

    assert_eq!(
        design.regression_config_for_seed(first_seed),
        Err(SimulationError::InvalidConfiguration)
    );
    assert_eq!(
        design.regression_config_for_seed(last_seed),
        Err(SimulationError::InvalidConfiguration)
    );

    let before_reserved = first_seed - 1;
    assert_eq!(
        design
            .regression_config_for_seed(before_reserved)
            .expect("seed immediately before the reserved range")
            .seed(),
        before_reserved
    );
    let after_reserved = last_seed + 1;
    assert_eq!(
        design
            .regression_config_for_seed(after_reserved)
            .expect("seed immediately after the reserved range")
            .seed(),
        after_reserved
    );

    let ci_seed = 101;
    let ci_config = design
        .regression_config_for_seed(ci_seed)
        .expect("same DGP shape under a non-acceptance CI seed");
    assert_eq!(ci_config.seed(), ci_seed);
    assert_eq!(ci_config.event_count(), 9);
    assert_eq!(ci_config.documents_per_event(), 2);
    assert_eq!(ci_config.membership_targets(), 3);
    assert_eq!(ci_config.max_report_delay_hours(), 12);
    assert_eq!(ci_config.max_availability_delay_hours(), 6);
    assert_eq!(ci_config.missingness_rate_bps(), 0);
    assert_eq!(ci_config.relation_false_negative_bps(), 0);
    assert_eq!(ci_config.relation_false_positive_bps(), 500);
    assert_eq!(ci_config.revision_rate_bps(), 3_000);
    assert_eq!(ci_config.translation_rate_bps(), 3_000);
    assert_eq!(ci_config.template_copy_rate_bps(), 3_000);
    assert_eq!(ci_config.topic_dgp(), TopicDgpConfig::ci_default());
}

#[test]
fn coverage_calibration_v1_fingerprint_binds_the_complete_declared_scenario() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let fingerprint = design
        .scenario_fingerprint()
        .expect("declared scenario fingerprint");

    assert_eq!(fingerprint.len(), 64);
    assert!(
        fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert_eq!(
        fingerprint,
        "e5dd9280b1bb4d9255bfeb5c5c3bee01638cdbd1f495887c5f2e93349597735a"
    );
}

#[test]
fn coverage_calibration_v1_config_rejects_out_of_range_replication() {
    let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    assert_eq!(
        design.config_for_replication(design.attempted_replication_count()),
        Err(SimulationError::InvalidConfiguration)
    );
}
