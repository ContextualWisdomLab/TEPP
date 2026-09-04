use validation_core::{MonteCarloSummary, ValidationError, summarize_replications};

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
    let attainable = summary(0.75, 1.75);
    assert!(attainable.validate().is_ok());
    assert!(serde_json::to_string(&attainable).is_ok());

    // Every nearest-rank percentile is an observed retained replication. Because
    // sample SD is computed from represented-mean deviations with denominator
    // n - 1, every endpoint must satisfy |x - mean| <= SD * sqrt(n - 1).
    // Here the support radius is 0.5 * sqrt(3) < 1, so endpoint 2.0 is impossible.
    let impossible = summary(0.75, 2.0);
    assert_eq!(impossible.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&impossible).is_err());

    let payload = r#"{"replication_count":4,"mean":1.0,"standard_deviation":0.5,"standard_error":0.25,"percentile_lower":0.75,"percentile_upper":2.0}"#;
    assert!(serde_json::from_str::<MonteCarloSummary>(payload).is_err());

    // Mean projection can round between adjacent binary64 observations. The
    // support check must therefore use the recorded squared-deviation identity,
    // not a stronger zero-sum-deviation bound that assumes an exact real mean.
    let adjacent = f64::from_bits(1.0_f64.to_bits() + 1);
    let rounded_mean = summarize_replications(&[1.0, adjacent], 0.0, 1.0)
        .expect("rounded mean sample remains admissible");
    assert_eq!(rounded_mean.percentile_upper.to_bits(), adjacent.to_bits());

    // The moment-support law is generic: signed scalar summaries remain valid
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
