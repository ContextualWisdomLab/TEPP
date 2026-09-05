use validation_core::{MonteCarloSummary, ValidationError};

fn summary(
    replication_count: usize,
    mean: f64,
    percentile_lower: f64,
    percentile_upper: f64,
) -> MonteCarloSummary {
    MonteCarloSummary {
        replication_count,
        mean,
        standard_deviation: 0.0,
        standard_error: 0.0,
        percentile_lower,
        percentile_upper,
    }
}

#[test]
fn zero_sample_spread_requires_degenerate_empirical_support() {
    let impossible = summary(4, 1.0, 0.5, 1.5);
    assert_eq!(impossible.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&impossible).is_err());

    let payload = r#"{"replication_count":4,"mean":1.0,"standard_deviation":0.0,"standard_error":0.0,"percentile_lower":0.5,"percentile_upper":1.5}"#;
    assert!(serde_json::from_str::<MonteCarloSummary>(payload).is_err());

    let impossible_singleton = summary(1, 2.0, 1.0, 2.0);
    assert_eq!(
        impossible_singleton.validate(),
        Err(ValidationError::InvalidInput)
    );

    let constant_signed = summary(4, -3.0, -3.0, -3.0);
    assert!(constant_signed.validate().is_ok());
    assert!(serde_json::to_string(&constant_signed).is_ok());

    let signed_zero = summary(4, -0.0, 0.0, -0.0);
    assert!(signed_zero.validate().is_ok());
}
