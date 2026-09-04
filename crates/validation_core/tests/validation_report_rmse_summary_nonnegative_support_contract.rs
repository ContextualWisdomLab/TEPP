use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

fn report_with(summary: MonteCarloSummary) -> ValidationReport {
    ValidationReport {
        study_label: "rmse-summary-nonnegative-support".into(),
        rmse: 0.2,
        rmse_standard_error: 0.05,
        mean_bias: 0.0,
        bias_standard_error: 0.0,
        interval_coverage: 0.5,
        coverage_wilson_lower: 0.2,
        coverage_wilson_upper: 0.8,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: Some(summary),
    }
}

#[test]
fn monte_carlo_rmse_rejects_spread_impossible_for_nonnegative_replications() {
    // For nonnegative replication metrics x_i with sample mean m,
    // sample SD is at most sqrt(n) * m. Equality occurs when one replication
    // carries the entire finite sum and the remaining n-1 replications are zero.
    // Therefore SE(mean) = SD / sqrt(n) cannot exceed m.
    let impossible_rmse_summary = MonteCarloSummary {
        replication_count: 4,
        mean: 1.0,
        standard_deviation: 3.0,
        standard_error: 1.5,
        percentile_lower: 0.0,
        percentile_upper: 4.0,
    };

    // The generic carrier is intentionally sign-neutral because it also serves
    // signed metrics such as bias. The stronger support belongs only to the
    // typed monte_carlo_rmse slot.
    assert!(impossible_rmse_summary.validate().is_ok());

    let report = report_with(impossible_rmse_summary);
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
    assert_eq!(report.to_json(), Err(ValidationError::InvalidInput));
    assert_eq!(
        report.to_human_summary(),
        Err(ValidationError::InvalidInput)
    );

    let payload = r#"{\"study_label\":\"rmse-summary-nonnegative-support\",\"rmse\":0.2,\"rmse_standard_error\":0.05,\"mean_bias\":0.0,\"bias_standard_error\":0.0,\"interval_coverage\":0.5,\"coverage_wilson_lower\":0.0,\"coverage_wilson_upper\":0.8,\"temporal_order_accuracy\":1.0,\"monte_carlo_rmse\":{\"replication_count\":4,\"mean\":1.0,\"standard_deviation\":3.0,\"standard_error\":1.5,\"percentile_lower\":0.0,\"percentile_upper\":4.0}}"#;
    assert!(serde_json::from_str::<ValidationReport>(payload).is_err());
}

#[test]
fn monte_carlo_rmse_accepts_attainable_nonnegative_support_boundary() {
    let boundary = MonteCarloSummary {
        replication_count: 4,
        mean: 1.0,
        standard_deviation: 2.0,
        standard_error: 1.0,
        percentile_lower: 0.0,
        percentile_upper: 4.0,
    };

    assert!(boundary.validate().is_ok());
    assert!(report_with(boundary).validate().is_ok());
}
