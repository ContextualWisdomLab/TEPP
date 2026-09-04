use validation_core::{
    ValidationError, ValidationReport, interval_coverage, wilson_coverage_interval,
};

fn base_report() -> ValidationReport {
    ValidationReport {
        study_label: "wilson-pair-coherence".into(),
        rmse: 0.2,
        rmse_standard_error: 0.05,
        mean_bias: 0.0,
        bias_standard_error: 0.01,
        interval_coverage: 0.5,
        coverage_wilson_lower: 0.2,
        coverage_wilson_upper: 0.9,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: None,
    }
}

#[test]
fn canonical_wilson_pair_remains_admissible() {
    let truth = [0.0, 1.0, 2.0, 3.0];
    let lower = [-1.0, 0.0, 3.0, 4.0];
    let upper = [1.0, 2.0, 4.0, 5.0];
    let coverage = interval_coverage(&truth, &lower, &upper).expect("coverage");
    assert_eq!(coverage, 0.5);
    let (wilson_lower, wilson_upper) =
        wilson_coverage_interval(&truth, &lower, &upper, 1.96).expect("wilson");

    let report = ValidationReport {
        interval_coverage: coverage,
        coverage_wilson_lower: wilson_lower,
        coverage_wilson_upper: wilson_upper,
        ..base_report()
    };

    assert_eq!(report.validate(), Ok(()));
    assert!(report.to_json().is_ok());
    assert!(report.to_human_summary().is_ok());
}

#[test]
fn complementary_identity_accepts_non_symmetric_canonical_pair() {
    let truth = [0.0, 1.0, 2.0, 3.0];
    let lower = [-1.0, 2.0, 3.0, 4.0];
    let upper = [1.0, 3.0, 4.0, 5.0];
    let coverage = interval_coverage(&truth, &lower, &upper).expect("coverage");
    assert_eq!(coverage, 0.25);
    let (wilson_lower, wilson_upper) =
        wilson_coverage_interval(&truth, &lower, &upper, 1.96).expect("wilson");

    let report = ValidationReport {
        interval_coverage: coverage,
        coverage_wilson_lower: wilson_lower,
        coverage_wilson_upper: wilson_upper,
        ..base_report()
    };
    assert_eq!(report.validate(), Ok(()));
}

#[test]
fn impossible_wilson_pair_fails_closed_across_report_boundaries() {
    // At p = 0.5 every Wilson score interval is symmetric about 0.5 for every
    // finite positive z and non-empty denominator. [0.2, 0.9] contains p but
    // cannot be emitted by the canonical Wilson producer for that proportion.
    let report = base_report();

    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
    assert_eq!(report.to_json(), Err(ValidationError::InvalidInput));
    assert_eq!(
        report.to_human_summary(),
        Err(ValidationError::InvalidInput)
    );

    let raw = r#"{
        "study_label":"wilson-pair-coherence",
        "rmse":0.2,
        "rmse_standard_error":0.05,
        "mean_bias":0.0,
        "bias_standard_error":0.01,
        "interval_coverage":0.5,
        "coverage_wilson_lower":0.2,
        "coverage_wilson_upper":0.9,
        "temporal_order_accuracy":1.0,
        "monte_carlo_rmse":null
    }"#;
    assert!(serde_json::from_str::<ValidationReport>(raw).is_err());
}
