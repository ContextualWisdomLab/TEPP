//! RMSE-slot validation contracts for otherwise valid scalar Monte Carlo summaries.

use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

fn report_with(summary: MonteCarloSummary) -> ValidationReport {
    ValidationReport {
        study_label: "rmse-support-contract".into(),
        rmse: 1.0,
        rmse_standard_error: 0.0,
        mean_bias: 0.0,
        bias_standard_error: 0.0,
        interval_coverage: 0.0,
        coverage_wilson_lower: 0.0,
        coverage_wilson_upper: 1.0,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: Some(summary),
    }
}

#[test]
fn rmse_slot_rejects_generic_signed_scalar_support() {
    let summary = MonteCarloSummary {
        replication_count: 2,
        mean: 0.0,
        standard_deviation: 2.0_f64.sqrt(),
        standard_error: 1.0,
        percentile_lower: -1.0,
        percentile_upper: 1.0,
    };
    summary
        .validate()
        .expect("signed scalar summary is generically coherent");

    assert_eq!(
        report_with(summary).validate(),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn rmse_slot_rejects_zero_mean_positive_spread() {
    let summary = MonteCarloSummary {
        replication_count: 4,
        mean: 0.0,
        standard_deviation: 1.0,
        standard_error: 0.5,
        percentile_lower: 0.0,
        percentile_upper: 0.0,
    };
    summary
        .validate()
        .expect("generic summary permits equal percentile endpoints inside positive spread");

    assert_eq!(
        report_with(summary).validate(),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn rmse_slot_rejects_standard_error_beyond_nonnegative_support() {
    let summary = MonteCarloSummary {
        replication_count: 4,
        mean: 1.0,
        standard_deviation: 4.0,
        standard_error: 2.0,
        percentile_lower: 1.0,
        percentile_upper: 1.0,
    };
    summary
        .validate()
        .expect("generic summary is coherent before applying RMSE support");

    assert_eq!(
        report_with(summary).validate(),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn rmse_slot_rejects_percentile_beyond_replication_sum_support() {
    let summary = MonteCarloSummary {
        replication_count: 4,
        mean: 1.0,
        standard_deviation: 2.0,
        standard_error: 1.0,
        percentile_lower: 1.0,
        percentile_upper: 4.1,
    };
    summary
        .validate()
        .expect("generic summary is coherent before applying RMSE support");

    assert_eq!(
        report_with(summary).validate(),
        Err(ValidationError::InvalidInput)
    );
}
