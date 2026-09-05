use validation_core::{ValidationError, ValidationReport, wilson_coverage_interval};

fn all_covered_report(wilson_lower: f64) -> ValidationReport {
    ValidationReport {
        study_label: "wilson-all-covered-positive-lower".into(),
        rmse: 0.2,
        rmse_standard_error: 0.05,
        mean_bias: 0.0,
        bias_standard_error: 0.01,
        interval_coverage: 1.0,
        coverage_wilson_lower: wilson_lower,
        coverage_wilson_upper: 1.0,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: None,
    }
}

#[test]
fn canonical_all_covered_wilson_lower_remains_strictly_positive() {
    let truth = [0.0];
    let lower = [-1.0];
    let upper = [1.0];
    let (wilson_lower, wilson_upper) =
        wilson_coverage_interval(&truth, &lower, &upper, 1.0e154).expect("wilson");

    assert!(wilson_lower > 0.0);
    assert_eq!(wilson_upper, 1.0);
    assert_eq!(all_covered_report(wilson_lower).validate(), Ok(()));
}

#[test]
fn zero_lower_all_covered_wilson_artifact_fails_closed() {
    // For p = 1 the canonical producer returns n / (n + z^2). With a non-empty
    // sample, finite z^2, and n >= 1, that represented lower endpoint is always
    // strictly positive. A stored [0, 1] pair therefore cannot come from the
    // producer even though it is ordered and contains the empirical coverage.
    for impossible_lower in [0.0, -0.0] {
        let report = all_covered_report(impossible_lower);
        assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
        assert_eq!(report.to_json(), Err(ValidationError::InvalidInput));
        assert_eq!(
            report.to_human_summary(),
            Err(ValidationError::InvalidInput)
        );
    }

    let raw = r#"{
        "study_label":"wilson-all-covered-positive-lower",
        "rmse":0.2,
        "rmse_standard_error":0.05,
        "mean_bias":0.0,
        "bias_standard_error":0.01,
        "interval_coverage":1.0,
        "coverage_wilson_lower":0.0,
        "coverage_wilson_upper":1.0,
        "temporal_order_accuracy":1.0,
        "monte_carlo_rmse":null
    }"#;
    assert!(serde_json::from_str::<ValidationReport>(raw).is_err());
}
