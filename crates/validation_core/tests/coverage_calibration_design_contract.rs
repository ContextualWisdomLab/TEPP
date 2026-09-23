use validation_core::{
    CoverageCalibrationDesign, assess_coverage_calibration,
    summarize_windowed_coverage_recovery_replications,
};

#[test]
fn tepp_nominal_95_design_is_versioned_before_the_larger_run() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();

    assert_eq!(design.design_id(), "tepp.coverage.nominal95.v1");
    assert_eq!(design.attempted_dgp_count(), 10_000);
    assert!((design.nominal_coverage() - 0.95).abs() < f64::EPSILON);
    assert!((design.practical_lower_coverage() - 0.91).abs() < f64::EPSILON);
    assert!((design.practical_upper_coverage() - 0.98).abs() < f64::EPSILON);
    assert!((design.maximum_monte_carlo_standard_error() - 0.005).abs() < f64::EPSILON);
}

#[test]
fn calibration_assessment_requires_the_predeclared_attempt_count_band_and_precision() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let successful = vec![vec![0.95]; 10_000];
    let summary = summarize_windowed_coverage_recovery_replications(
        10_000,
        &successful,
        0.025,
        0.975,
    )
    .expect("predeclared successful calibration summary");

    let assessment = assess_coverage_calibration(&design, &summary)
        .expect("matching attempted-DGP design");
    assert!(assessment.coverage_within_practical_band());
    assert!(assessment.monte_carlo_precision_sufficient());
    assert!(assessment.supports_calibration_claim());
    assert_eq!(assessment.attempted_replication_count(), 10_000);
    assert_eq!(assessment.successful_replication_count(), 10_000);
    assert_eq!(assessment.failure_count(), 0);
}

#[test]
fn calibration_assessment_refuses_post_hoc_size_substitution_and_imprecise_success() {
    let design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let short_summary = summarize_windowed_coverage_recovery_replications(
        9_999,
        &vec![vec![0.95]; 9_999],
        0.025,
        0.975,
    )
    .expect("valid but wrong-size experiment");
    assert!(assess_coverage_calibration(&design, &short_summary).is_err());

    let one_success = summarize_windowed_coverage_recovery_replications(
        10_000,
        &[vec![0.95]],
        0.025,
        0.975,
    )
    .expect("denominator-preserving singleton success");
    let assessment = assess_coverage_calibration(&design, &one_success)
        .expect("attempt count matches the predeclared design");
    assert!(assessment.coverage_within_practical_band());
    assert!(!assessment.monte_carlo_precision_sufficient());
    assert!(!assessment.supports_calibration_claim());
    assert_eq!(assessment.failure_count(), 9_999);
}
