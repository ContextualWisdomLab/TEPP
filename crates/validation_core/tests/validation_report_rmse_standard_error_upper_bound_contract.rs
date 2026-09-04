use validation_core::{
    ValidationError, ValidationReport, rmse_standard_error, root_mean_square_error,
};

fn report_with(rmse: f64, rmse_standard_error: f64) -> ValidationReport {
    ValidationReport {
        study_label: "rmse-se-support".into(),
        rmse,
        rmse_standard_error,
        mean_bias: 0.0,
        bias_standard_error: 0.0,
        interval_coverage: 0.95,
        coverage_wilson_lower: 0.90,
        coverage_wilson_upper: 0.98,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: None,
    }
}

#[test]
fn report_rejects_rmse_standard_error_above_squared_residual_support_bound() {
    // For x_i = r_i^2 >= 0 with sample SD in the crate's delta-method producer,
    // sd(x) <= sqrt(n) * mean(x), hence SE(RMSE) <= RMSE / 2.
    // Two residuals [0, 1] attain the mathematical boundary.
    let truth = [0.0, 0.0];
    let recovered = [0.0, 1.0];
    let rmse = root_mean_square_error(&truth, &recovered).expect("rmse");
    let rmse_se = rmse_standard_error(&truth, &recovered).expect("rmse se");
    let canonical = report_with(rmse, rmse_se);
    assert!(canonical.validate().is_ok());
    assert!(rmse_se / rmse <= 0.5 + 64.0 * f64::EPSILON);

    let impossible = report_with(0.2, 0.11);
    assert_eq!(impossible.validate(), Err(ValidationError::InvalidInput));
    assert_eq!(impossible.to_json(), Err(ValidationError::InvalidInput));
    assert_eq!(
        impossible.to_human_summary(),
        Err(ValidationError::InvalidInput)
    );

    let ingress = r#"{"study_label":"rmse-se-support","rmse":0.2,"rmse_standard_error":0.11,"mean_bias":0.0,"bias_standard_error":0.0,"interval_coverage":0.95,"coverage_wilson_lower":0.9,"coverage_wilson_upper":0.98,"temporal_order_accuracy":1.0,"monte_carlo_rmse":null}"#;
    assert!(serde_json::from_str::<ValidationReport>(ingress).is_err());
}
