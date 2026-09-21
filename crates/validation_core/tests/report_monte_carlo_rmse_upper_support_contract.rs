//! RMSE-specific upper-support regression for a generically valid signed summary.

use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

#[test]
fn report_rejects_monte_carlo_rmse_upper_percentile_above_nonnegative_sample_sum() {
    let summary = MonteCarloSummary {
        replication_count: 3,
        mean: 1.0,
        standard_deviation: 1.7,
        standard_error: 1.7 / 3.0_f64.sqrt(),
        percentile_lower: 1.0,
        percentile_upper: 3.1,
    };

    // The generic summary contract permits signed metrics and therefore cannot
    // impose nonnegative RMSE support by itself.
    assert_eq!(summary.validate(), Ok(summary));

    let report = ValidationReport {
        study_label: "rmse-upper-support".into(),
        rmse: 0.1,
        rmse_standard_error: 0.01,
        mean_bias: 0.0,
        bias_standard_error: 0.02,
        interval_coverage: 0.5,
        coverage_wilson_lower: 0.2,
        coverage_wilson_upper: 0.8,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: Some(summary),
    };

    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
}
