use validation_core::{MonteCarloSummary, ValidationError, ValidationReport};

fn valid_report() -> ValidationReport {
    ValidationReport {
        study_label: "validation-report-contract".into(),
        rmse: 0.1,
        rmse_standard_error: 0.01,
        mean_bias: -0.02,
        bias_standard_error: 0.02,
        interval_coverage: 0.5,
        coverage_wilson_lower: 0.2,
        coverage_wilson_upper: 0.8,
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
    report.coverage_wilson_upper = 0.49;
    assert_eq!(report.validate(), Err(ValidationError::InvalidInput));
}

#[test]
fn every_report_projection_enforces_validation() {
    let mut report = valid_report();
    report.interval_coverage = 1.5;
    assert_eq!(report.to_json(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&report).is_err());
    assert_eq!(
        report.to_human_summary(),
        Err(ValidationError::InvalidInput)
    );

    let impossible = r#"{
        "study_label":"invalid-wire-report",
        "rmse":0.1,
        "rmse_standard_error":0.01,
        "mean_bias":0.0,
        "bias_standard_error":0.01,
        "interval_coverage":1.5,
        "coverage_wilson_lower":0.2,
        "coverage_wilson_upper":0.8,
        "temporal_order_accuracy":0.75,
        "monte_carlo_rmse":null
    }"#;
    assert!(serde_json::from_str::<ValidationReport>(impossible).is_err());
}

#[test]
fn monte_carlo_summary_direct_serialization_preserves_its_validation_contract() {
    let impossible = MonteCarloSummary {
        replication_count: 2,
        mean: 0.0,
        standard_deviation: -0.1,
        standard_error: 0.0,
        percentile_lower: 0.0,
        percentile_upper: 1.0,
    };
    assert_eq!(impossible.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&impossible).is_err());
}
