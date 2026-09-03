use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

fn report_with(summary: MonteCarloSummary) -> ValidationReport {
    ValidationReport {
        study_label: "zero-mean-rmse-summary".into(),
        rmse: 0.0,
        rmse_standard_error: 0.0,
        mean_bias: 0.0,
        bias_standard_error: 0.0,
        interval_coverage: 1.0,
        coverage_wilson_lower: 0.5,
        coverage_wilson_upper: 1.0,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: Some(summary),
    }
}

#[test]
fn zero_mean_monte_carlo_rmse_requires_zero_spread_and_zero_percentiles() {
    let positive_spread = MonteCarloSummary {
        replication_count: 4,
        mean: 0.0,
        standard_deviation: 1.0,
        standard_error: 0.5,
        percentile_lower: 0.0,
        percentile_upper: 1.0,
    };
    assert!(positive_spread.validate().is_ok());
    let report = report_with(positive_spread);
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
    assert_eq!(report.to_json(), Err(ValidationError::InvalidInput));
    assert_eq!(
        report.to_human_summary(),
        Err(ValidationError::InvalidInput)
    );

    let positive_percentile = MonteCarloSummary {
        replication_count: 4,
        mean: 0.0,
        standard_deviation: 0.0,
        standard_error: 0.0,
        percentile_lower: 0.0,
        percentile_upper: 1.0,
    };
    assert!(positive_percentile.validate().is_ok());
    assert_eq!(
        report_with(positive_percentile).validate(),
        Err(ValidationError::InvalidInput)
    );

    let perfect_recovery = MonteCarloSummary {
        replication_count: 4,
        mean: -0.0,
        standard_deviation: 0.0,
        standard_error: -0.0,
        percentile_lower: -0.0,
        percentile_upper: 0.0,
    };
    assert!(perfect_recovery.validate().is_ok());
    assert!(report_with(perfect_recovery).validate().is_ok());

    let payload = r#"{"study_label":"zero-mean-rmse-summary","rmse":0.0,"rmse_standard_error":0.0,"mean_bias":0.0,"bias_standard_error":0.0,"interval_coverage":1.0,"coverage_wilson_lower":0.5,"coverage_wilson_upper":1.0,"temporal_order_accuracy":1.0,"monte_carlo_rmse":{"replication_count":4,"mean":0.0,"standard_deviation":1.0,"standard_error":0.5,"percentile_lower":0.0,"percentile_upper":1.0}}"#;
    assert!(serde_json::from_str::<ValidationReport>(payload).is_err());
}
