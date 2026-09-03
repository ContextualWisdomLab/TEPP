use validation_core::{MonteCarloSummary, ValidationError};

fn summary(replication_count: usize, standard_deviation: f64, standard_error: f64) -> MonteCarloSummary {
    MonteCarloSummary {
        replication_count,
        mean: 0.5,
        standard_deviation,
        standard_error,
        percentile_lower: -2.0,
        percentile_upper: 3.0,
    }
}

#[test]
fn monte_carlo_summary_rejects_impossible_standard_error_evidence() {
    let larger_than_sd = summary(4, 0.5, 1.0);
    assert_eq!(larger_than_sd.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&larger_than_sd).is_err());

    let false_zero_uncertainty = summary(4, 0.5, 0.0);
    assert_eq!(
        false_zero_uncertainty.validate(),
        Err(ValidationError::InvalidInput)
    );

    let impossible_singleton_spread = summary(1, 0.5, 0.5);
    assert_eq!(
        impossible_singleton_spread.validate(),
        Err(ValidationError::InvalidInput)
    );

    let payload = r#"{"replication_count":4,"mean":0.5,"standard_deviation":0.5,"standard_error":1.0,"percentile_lower":-2.0,"percentile_upper":3.0}"#;
    assert!(serde_json::from_str::<MonteCarloSummary>(payload).is_err());
}
