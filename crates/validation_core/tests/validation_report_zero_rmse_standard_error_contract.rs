use validation_core::{
    ValidationError, ValidationReport, rmse_standard_error, root_mean_square_error,
};

fn report_with(rmse: f64, rmse_standard_error: f64) -> ValidationReport {
    ValidationReport {
        study_label: "zero-rmse-standard-error".into(),
        rmse,
        rmse_standard_error,
        mean_bias: 0.0,
        bias_standard_error: 0.0,
        interval_coverage: 1.0,
        coverage_wilson_lower: 0.5,
        coverage_wilson_upper: 1.0,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: None,
    }
}

#[test]
fn exact_zero_rmse_requires_exact_zero_rmse_standard_error() {
    let truth = [1.0, -2.0, 3.0];
    let recovered = truth;
    assert_eq!(root_mean_square_error(&truth, &recovered), Ok(0.0));
    assert_eq!(rmse_standard_error(&truth, &recovered), Ok(0.0));

    let impossible = report_with(0.0, 0.1);
    assert_eq!(impossible.validate(), Err(ValidationError::InvalidInput));
    assert_eq!(impossible.to_json(), Err(ValidationError::InvalidInput));
    assert_eq!(
        impossible.to_human_summary(),
        Err(ValidationError::InvalidInput)
    );

    let signed_zero = report_with(-0.0, -0.0);
    assert!(signed_zero.validate().is_ok());

    let payload = r#"{"study_label":"zero-rmse-standard-error","rmse":0.0,"rmse_standard_error":0.1,"mean_bias":0.0,"bias_standard_error":0.0,"interval_coverage":1.0,"coverage_wilson_lower":0.5,"coverage_wilson_upper":1.0,"temporal_order_accuracy":1.0,"monte_carlo_rmse":null}"#;
    assert!(serde_json::from_str::<ValidationReport>(payload).is_err());
}
