use validation_core::{MonteCarloSummary, ValidationError};

fn inconsistent_standard_error_summary() -> MonteCarloSummary {
    MonteCarloSummary {
        replication_count: 4,
        mean: 0.5,
        standard_deviation: 2.0,
        standard_error: 0.5,
        percentile_lower: -2.0,
        percentile_upper: 3.0,
    }
}

#[test]
fn monte_carlo_summary_rejects_standard_error_that_disagrees_with_sd_and_n() {
    let summary = inconsistent_standard_error_summary();

    assert_eq!(summary.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&summary).is_err());

    let payload = r#"{"replication_count":4,"mean":0.5,"standard_deviation":2.0,"standard_error":0.5,"percentile_lower":-2.0,"percentile_upper":3.0}"#;
    assert!(serde_json::from_str::<MonteCarloSummary>(payload).is_err());
}
