use validation_core::{ValidationError, ValidationReport};

fn valid_report() -> ValidationReport {
    ValidationReport {
        study_label: "validation-report-contract".into(),
        rmse: 0.1,
        rmse_standard_error: 0.01,
        mean_bias: -0.02,
        bias_standard_error: 0.02,
        interval_coverage: 0.8,
        coverage_wilson_lower: 0.6,
        coverage_wilson_upper: 0.9,
        temporal_order_accuracy: 0.75,
        monte_carlo_rmse: None,
    }
}

#[test]
fn validation_report_rejects_impossible_metric_domains() {
    let mut report = valid_report();
    report.rmse = -0.1;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));

    let mut report = valid_report();
    report.rmse_standard_error = -0.01;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));

    let mut report = valid_report();
    report.bias_standard_error = -0.01;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));

    for invalid_coverage in [-0.01, 1.01] {
        let mut report = valid_report();
        report.interval_coverage = invalid_coverage;
        assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
    }

    for invalid_accuracy in [-0.01, 1.01] {
        let mut report = valid_report();
        report.temporal_order_accuracy = invalid_accuracy;
        assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
    }
}

#[test]
fn validation_report_rejects_incoherent_wilson_evidence() {
    let mut report = valid_report();
    report.coverage_wilson_lower = -0.01;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));

    let mut report = valid_report();
    report.coverage_wilson_upper = 1.01;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));

    let mut report = valid_report();
    report.coverage_wilson_lower = 0.95;
    report.coverage_wilson_upper = 0.90;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));

    let mut report = valid_report();
    report.coverage_wilson_lower = 0.81;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));

    let mut report = valid_report();
    report.coverage_wilson_upper = 0.79;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
}

#[test]
fn serialization_cannot_bypass_report_validation() {
    let mut report = valid_report();
    report.interval_coverage = 1.5;
    assert_eq!(report.to_json(), Err(ValidationError::InvalidInput));
}
