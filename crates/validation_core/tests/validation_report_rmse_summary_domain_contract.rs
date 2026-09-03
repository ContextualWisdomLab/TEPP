use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

fn report_with_rmse_summary(summary: MonteCarloSummary) -> ValidationReport {
    ValidationReport {
        study_label: "rmse-domain-contract".into(),
        rmse: 0.2,
        rmse_standard_error: 0.01,
        mean_bias: 0.0,
        bias_standard_error: 0.01,
        interval_coverage: 0.95,
        coverage_wilson_lower: 0.85,
        coverage_wilson_upper: 0.99,
        temporal_order_accuracy: 0.9,
        monte_carlo_rmse: Some(summary),
    }
}

#[test]
fn validation_report_rejects_negative_monte_carlo_rmse_evidence() {
    let negative_mean = report_with_rmse_summary(MonteCarloSummary {
        replication_count: 20,
        mean: -0.1,
        standard_deviation: 0.02,
        standard_error: 0.004,
        percentile_lower: -0.14,
        percentile_upper: -0.06,
    });
    assert_eq!(negative_mean.validate(), Err(ValidationError::InvalidInput));
    assert_eq!(negative_mean.to_json(), Err(ValidationError::InvalidInput));
    assert_eq!(
        negative_mean.to_human_summary(),
        Err(ValidationError::InvalidInput)
    );

    let negative_percentile = report_with_rmse_summary(MonteCarloSummary {
        replication_count: 20,
        mean: 0.02,
        standard_deviation: 0.03,
        standard_error: 0.006,
        percentile_lower: -0.01,
        percentile_upper: 0.07,
    });
    assert_eq!(
        negative_percentile.validate(),
        Err(ValidationError::InvalidInput)
    );

    let serialized = r#"{
        "study_label":"rmse-domain-contract",
        "rmse":0.2,
        "rmse_standard_error":0.01,
        "mean_bias":0.0,
        "bias_standard_error":0.01,
        "interval_coverage":0.95,
        "coverage_wilson_lower":0.85,
        "coverage_wilson_upper":0.99,
        "temporal_order_accuracy":0.9,
        "monte_carlo_rmse":{
            "replication_count":20,
            "mean":-0.1,
            "standard_deviation":0.02,
            "standard_error":0.004,
            "percentile_lower":-0.14,
            "percentile_upper":-0.06
        }
    }"#;
    assert!(serde_json::from_str::<ValidationReport>(serialized).is_err());
}
