use validation_core::{MonteCarloSummary, ValidationError};

fn summary(percentile_lower: f64, percentile_upper: f64) -> MonteCarloSummary {
    MonteCarloSummary {
        replication_count: 4,
        mean: 1.0,
        standard_deviation: 0.5,
        standard_error: 0.25,
        percentile_lower,
        percentile_upper,
    }
}

#[test]
fn empirical_percentiles_must_fit_recorded_mean_and_sample_spread() {
    // [0.75, 0.75, 0.75, 1.75] attains this finite-sample support boundary:
    // mean = 1, sample SD = 0.5, and max deviation = SD * (n - 1) / sqrt(n) = 0.75.
    let attainable = summary(0.75, 1.75);
    assert!(attainable.validate().is_ok());
    assert!(serde_json::to_string(&attainable).is_ok());

    // Every nearest-rank percentile is an observed retained replication. No sample
    // with n = 4, mean = 1, and sample SD = 0.5 can contain 2.0, because its
    // deviation from the mean exceeds the finite-sample support bound above.
    let impossible = summary(0.75, 2.0);
    assert_eq!(impossible.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&impossible).is_err());

    let payload = r#"{"replication_count":4,"mean":1.0,"standard_deviation":0.5,"standard_error":0.25,"percentile_lower":0.75,"percentile_upper":2.0}"#;
    assert!(serde_json::from_str::<MonteCarloSummary>(payload).is_err());

    // The same moment-support law is generic: signed scalar summaries remain valid
    // when their empirical endpoints fit the represented mean and sample spread.
    let signed = MonteCarloSummary {
        replication_count: 4,
        mean: -1.0,
        standard_deviation: 0.5,
        standard_error: 0.25,
        percentile_lower: -1.75,
        percentile_upper: -0.75,
    };
    assert!(signed.validate().is_ok());
}
