use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

fn report_with(summary: MonteCarloSummary) -> ValidationReport {
    ValidationReport {
        study_label: "rmse-summary-percentile-support".into(),
        rmse: 0.2,
        rmse_standard_error: 0.05,
        mean_bias: 0.0,
        bias_standard_error: 0.0,
        interval_coverage: 0.95,
        coverage_wilson_lower: 0.8,
        coverage_wilson_upper: 1.0,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: Some(summary),
    }
}

#[test]
fn monte_carlo_rmse_rejects_percentile_above_nonnegative_sample_sum_support() {
    // For n nonnegative RMSE replications with represented mean m, every retained
    // replication is bounded by the finite sample sum n*m. Any empirical nearest-rank
    // percentile is one of those retained values and therefore cannot exceed n*m.
    let impossible_rmse_summary = MonteCarloSummary {
        replication_count: 4,
        mean: 1.0,
        standard_deviation: 0.5,
        standard_error: 0.25,
        percentile_lower: 0.0,
        percentile_upper: 5.0,
    };

    // The generic carrier cannot impose this bound because it also summarizes
    // signed metrics. The stronger support belongs to the typed RMSE slot.
    assert!(impossible_rmse_summary.validate().is_ok());

    let report = report_with(impossible_rmse_summary);
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
    assert_eq!(report.to_json(), Err(ValidationError::InvalidInput));
    assert_eq!(
        report.to_human_summary(),
        Err(ValidationError::InvalidInput)
    );

    let payload = r#"{"study_label":"rmse-summary-percentile-support","rmse":0.2,"rmse_standard_error":0.05,"mean_bias":0.0,"bias_standard_error":0.0,"interval_coverage":0.95,"coverage_wilson_lower":0.8,"coverage_wilson_upper":1.0,"temporal_order_accuracy":1.0,"monte_carlo_rmse":{"replication_count":4,"mean":1.0,"standard_deviation":0.5,"standard_error":0.25,"percentile_lower":0.0,"percentile_upper":5.0}}"#;
    assert!(serde_json::from_str::<ValidationReport>(payload).is_err());
}

#[test]
fn monte_carlo_rmse_accepts_empirical_percentile_at_nonnegative_sum_boundary() {
    // [0, 0, 0, 4] attains max(x_i) = n*mean and also the existing SD/SE support
    // boundary, so the endpoint must remain admissible rather than being tightened
    // by an arbitrary heuristic.
    let attainable_boundary = MonteCarloSummary {
        replication_count: 4,
        mean: 1.0,
        standard_deviation: 2.0,
        standard_error: 1.0,
        percentile_lower: 0.0,
        percentile_upper: 4.0,
    };

    assert!(attainable_boundary.validate().is_ok());
    assert!(report_with(attainable_boundary).validate().is_ok());
}
