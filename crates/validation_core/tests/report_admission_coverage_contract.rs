//! Edge contracts for Validation Report scientific-admission refusals.

use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

fn valid_report() -> ValidationReport {
    ValidationReport {
        study_label: "report-admission-contract".into(),
        rmse: 1.0,
        rmse_standard_error: 0.1,
        mean_bias: 0.0,
        bias_standard_error: 0.1,
        interval_coverage: 0.5,
        coverage_wilson_lower: 0.2,
        coverage_wilson_upper: 0.8,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: None,
    }
}

fn assert_invalid(report: ValidationReport) {
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
}

#[test]
fn point_rmse_refuses_nonfinite_relative_standard_error() {
    let mut report = valid_report();
    report.rmse = f64::from_bits(1);
    report.rmse_standard_error = 1.0;
    assert_invalid(report);
}

#[test]
fn probability_fields_and_wilson_identity_fail_closed_independently() {
    for mutate in [
        |report: &mut ValidationReport| report.interval_coverage = 1.1,
        |report: &mut ValidationReport| report.coverage_wilson_lower = -0.1,
        |report: &mut ValidationReport| report.coverage_wilson_upper = 1.1,
        |report: &mut ValidationReport| report.temporal_order_accuracy = -0.1,
        |report: &mut ValidationReport| {
            report.coverage_wilson_lower = 0.8;
            report.coverage_wilson_upper = 0.2;
        },
        |report: &mut ValidationReport| {
            report.interval_coverage = 0.1;
            report.coverage_wilson_lower = 0.2;
        },
        |report: &mut ValidationReport| {
            report.interval_coverage = 0.9;
            report.coverage_wilson_upper = 0.8;
        },
        |report: &mut ValidationReport| {
            report.coverage_wilson_lower = 0.1;
            report.coverage_wilson_upper = 0.8;
        },
    ] {
        let mut report = valid_report();
        mutate(&mut report);
        assert_invalid(report);
    }
}

#[test]
fn rmse_monte_carlo_slot_refuses_signed_and_impossible_nonnegative_support() {
    let cases = [
        MonteCarloSummary {
            replication_count: 3,
            mean: -1.0,
            standard_deviation: 0.2,
            standard_error: 0.2 / 3.0_f64.sqrt(),
            percentile_lower: -1.2,
            percentile_upper: -0.8,
        },
        MonteCarloSummary {
            replication_count: 3,
            mean: 0.0,
            standard_deviation: 1.0,
            standard_error: 1.0 / 3.0_f64.sqrt(),
            percentile_lower: 0.0,
            percentile_upper: 1.0,
        },
        MonteCarloSummary {
            replication_count: 4,
            mean: 0.1,
            standard_deviation: 1.0,
            standard_error: 0.5,
            percentile_lower: 0.0,
            percentile_upper: 1.0,
        },
        MonteCarloSummary {
            replication_count: 5,
            mean: 1.0,
            standard_deviation: 2.21,
            standard_error: 2.21 / 5.0_f64.sqrt(),
            percentile_lower: 1.0,
            percentile_upper: 5.4,
        },
    ];

    for summary in cases {
        summary.validate().expect("generic signed-metric summary");
        let mut report = valid_report();
        report.monte_carlo_rmse = Some(summary);
        assert_invalid(report);
    }
}
