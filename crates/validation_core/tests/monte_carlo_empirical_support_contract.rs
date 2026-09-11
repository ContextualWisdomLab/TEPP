//! Empirical-support regressions for externally constructed Monte Carlo summaries.

use validation_core::{MonteCarloSummary, ValidationError, summarize_replications};

fn three_replication_summary(lower: f64, upper: f64) -> MonteCarloSummary {
    MonteCarloSummary {
        replication_count: 3,
        mean: 0.0,
        standard_deviation: 1.0,
        standard_error: 1.0 / 3.0_f64.sqrt(),
        percentile_lower: lower,
        percentile_upper: upper,
    }
}

#[test]
fn summary_rejects_endpoint_outside_recorded_moment_support() {
    let summary = three_replication_summary(-3.0, 0.0);

    assert_eq!(summary.validate(), Err(ValidationError::InvalidInput));
}

#[test]
fn summary_rejects_distinct_endpoints_that_exceed_joint_deviation_budget() {
    let summary = three_replication_summary(-1.2, 1.2);

    // Each endpoint separately fits SD * sqrt(n - 1); the pair does not fit
    // the shared (n - 1) * SD^2 deviation budget.
    assert_eq!(summary.validate(), Err(ValidationError::InvalidInput));
}

#[test]
fn summarization_refuses_unrepresentable_full_range_sample_spread() {
    assert_eq!(
        summarize_replications(&[-f64::MAX, f64::MAX], 0.0, 1.0),
        Err(ValidationError::InvalidInput)
    );
}
