//! RMSE-specific support regression for a generically valid zero-mean signed summary.

use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

#[test]
fn report_rejects_zero_mean_monte_carlo_rmse_with_positive_spread() {
    let report = ValidationReport {
        study_label: "zero-mean-positive-spread".into(),
        rmse: 0.1,
        rmse_standard_error: 0.01,
        mean_bias: 0.0,
        bias_standard_error: 0.02,
        interval_coverage: 0.5,
        coverage_wilson_lower: 0.2,
        coverage_wilson_upper: 0.8,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: Some(MonteCarloSummary {
            replication_count: 3,
            mean: 0.0,
            standard_deviation: 1.0,
            standard_error: 1.0 / 3.0_f64.sqrt(),
            percentile_lower: 0.0,
            percentile_upper: 0.0,
        }),
    };

    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
}
