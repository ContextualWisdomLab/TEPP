use validation_core::{MonteCarloSummary, ValidationError, summarize_replications};

#[test]
fn distinct_percentile_endpoints_exhaust_a_two_replication_sample() {
    let attainable = summarize_replications(&[-0.5, 0.5], 0.5, 1.0)
        .expect("two-replication nearest-rank summary");
    assert_eq!(attainable.replication_count, 2);
    assert_eq!(attainable.mean, 0.0);
    assert_eq!(attainable.percentile_lower, -0.5);
    assert_eq!(attainable.percentile_upper, 0.5);
    assert!(attainable.validate().is_ok());

    // With exactly two retained replications, two distinct nearest-rank endpoint
    // values exhaust the sample: there are no unobserved replications left that
    // could supply additional spread. These endpoint values imply sample
    // SD = sqrt(0.5), so recording SD = 1.0 is scientifically impossible even
    // though each endpoint separately and jointly fits the looser moment budget.
    let impossible = MonteCarloSummary {
        replication_count: 2,
        mean: 0.0,
        standard_deviation: 1.0,
        standard_error: 1.0 / 2.0_f64.sqrt(),
        percentile_lower: -0.5,
        percentile_upper: 0.5,
    };
    assert_eq!(impossible.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&impossible).is_err());

    let payload = format!(
        "{{\"replication_count\":2,\"mean\":0.0,\"standard_deviation\":1.0,\"standard_error\":{},\"percentile_lower\":-0.5,\"percentile_upper\":0.5}}",
        1.0 / 2.0_f64.sqrt()
    );
    assert!(serde_json::from_str::<MonteCarloSummary>(&payload).is_err());
}
