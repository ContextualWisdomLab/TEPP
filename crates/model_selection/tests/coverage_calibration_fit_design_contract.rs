use model_selection::CoverageCalibrationFitDesign;

#[test]
fn coverage_calibration_fit_design_is_pinned_before_acceptance() {
    let design = CoverageCalibrationFitDesign::truth_k_v1();

    assert_eq!(
        design.design_id(),
        "tepp.model_selection.coverage_calibration_fit.truth_k.v1"
    );
    assert_eq!(
        design.candidate_strategy_id(),
        "tepp.model_selection.coverage_calibration_fit.candidate_strategy.truth_k_single_candidate.v1"
    );
    assert_eq!(design.seeds(), &[7, 11, 19]);
    assert_eq!(design.maximum_iterations(), 2_000);
    assert_eq!(design.tolerance().to_bits(), 0.001_f64.to_bits());
    assert_eq!(design.prior_variance().to_bits(), 1.0_f64.to_bits());
    assert_eq!(design.relation_strength().to_bits(), 0.25_f64.to_bits());
    assert_eq!(design.ridge().to_bits(), 0.01_f64.to_bits());
    assert_eq!(design.topic_smoothing().to_bits(), 0.05_f64.to_bits());
    assert_eq!(design.step_size().to_bits(), 0.2_f64.to_bits());

    let config = design
        .config_for_truth_k(3)
        .expect("truth-K coverage-calibration fit design");
    assert_eq!(config.candidate_topic_counts(), &[3]);
    assert_eq!(config.seeds(), design.seeds());
    assert_eq!(config.maximum_iterations(), design.maximum_iterations());
    assert_eq!(config.tolerance().to_bits(), design.tolerance().to_bits());

    assert_eq!(
        design
            .fingerprint_for_truth_k(3)
            .expect("canonical truth-K fit-design fingerprint"),
        "f7a353e89aa97a33a51b776406138a53fe5e776d1987c31fade270868bf55983"
    );
    assert_ne!(
        design
            .fingerprint_for_truth_k(2)
            .expect("candidate K participates in the fingerprint"),
        design
            .fingerprint_for_truth_k(3)
            .expect("candidate K participates in the fingerprint")
    );
    assert!(design.config_for_truth_k(1).is_err());
}
