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
fn distinct_empirical_percentiles_must_share_the_recorded_deviation_budget() {
    // [0.75, 0.75, 0.75, 1.75] has mean 1, sample SD 0.5, and
    // nearest-rank endpoints 0.75 and 1.75, so the joint support is attainable.
    let attainable = summary(0.75, 1.75);
    assert!(attainable.validate().is_ok());
    assert!(serde_json::to_string(&attainable).is_ok());

    // Each endpoint below is individually within SD * sqrt(n - 1):
    // |0.25 - 1| = |1.75 - 1| = 0.75 < 0.5 * sqrt(3).
    // They are nevertheless impossible together because distinct observed
    // endpoints consume at least 0.75^2 + 0.75^2 = 1.125 of squared-deviation
    // budget while the recorded sample SD permits only (n - 1) * SD^2 = 0.75.
    let impossible = summary(0.25, 1.75);
    assert_eq!(impossible.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&impossible).is_err());

    let payload = r#"{"replication_count":4,"mean":1.0,"standard_deviation":0.5,"standard_error":0.25,"percentile_lower":0.25,"percentile_upper":1.75}"#;
    assert!(serde_json::from_str::<MonteCarloSummary>(payload).is_err());

    // Equal percentile endpoints may designate the same retained observation,
    // so their squared deviation must not be counted twice.
    let same_rank = summary(1.75, 1.75);
    assert!(same_rank.validate().is_ok());
}
