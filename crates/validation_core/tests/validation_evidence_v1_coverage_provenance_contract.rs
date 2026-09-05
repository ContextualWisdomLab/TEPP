use validation_core::{
    ValidationError, ValidationEvidenceV1, ValidationReport, WilsonCoverageEvidenceV1,
};

fn canonical_coverage() -> WilsonCoverageEvidenceV1 {
    let truth = [0.0, 1.0, 2.0, 3.0];
    let lower = [-0.5, 0.5, 1.0, 4.0];
    let upper = [0.5, 1.5, 2.5, 5.0];
    WilsonCoverageEvidenceV1::from_intervals(&truth, &lower, &upper, 1.96)
        .expect("coverage evidence")
}

fn canonical_report(coverage: WilsonCoverageEvidenceV1) -> ValidationReport {
    ValidationReport {
        study_label: "wilson-provenance".into(),
        rmse: 0.1,
        rmse_standard_error: 0.01,
        mean_bias: 0.0,
        bias_standard_error: 0.02,
        interval_coverage: coverage.empirical_coverage,
        coverage_wilson_lower: coverage.wilson_lower,
        coverage_wilson_upper: coverage.wilson_upper,
        temporal_order_accuracy: 1.0,
        monte_carlo_rmse: None,
    }
}

#[test]
fn validation_evidence_v1_round_trips_report_and_wilson_provenance() {
    let coverage = canonical_coverage();
    let report = canonical_report(coverage);
    let evidence = ValidationEvidenceV1::new(report, coverage).expect("validation evidence");

    let json = evidence.to_json().expect("json");
    assert!(json.contains("\"schema\":\"tepp.validation_evidence.v1\""));
    assert!(json.contains("\"sample_count\":4"));
    assert!(json.contains("\"covered_count\":3"));
    assert!(json.contains("\"interval_sidedness\":\"two_sided\""));

    let decoded: ValidationEvidenceV1 = serde_json::from_str(&json).expect("decode");
    assert_eq!(decoded, evidence);
}

#[test]
fn report_projection_must_match_the_versioned_coverage_evidence() {
    let coverage = canonical_coverage();
    let mut report = canonical_report(coverage);
    report.interval_coverage = 0.5;
    assert_eq!(
        ValidationEvidenceV1::new(report, coverage),
        Err(ValidationError::InvalidInput)
    );

    let coverage = canonical_coverage();
    let mut report = canonical_report(coverage);
    report.coverage_wilson_lower = 0.0;
    assert_eq!(
        ValidationEvidenceV1::new(report, coverage),
        Err(ValidationError::InvalidInput)
    );

    let coverage = canonical_coverage();
    let mut report = canonical_report(coverage);
    report.coverage_wilson_upper = 1.0;
    assert_eq!(
        ValidationEvidenceV1::new(report, coverage),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn serde_rejects_schema_drift_and_nested_provenance_tampering() {
    let coverage = canonical_coverage();
    let evidence = ValidationEvidenceV1::new(canonical_report(coverage), coverage)
        .expect("validation evidence");
    let json = evidence.to_json().expect("json");

    let wrong_schema = json.replace("tepp.validation_evidence.v1", "tepp.validation_evidence.v2");
    assert!(serde_json::from_str::<ValidationEvidenceV1>(&wrong_schema).is_err());

    let wrong_count = json.replacen("\"sample_count\":4", "\"sample_count\":5", 1);
    assert!(serde_json::from_str::<ValidationEvidenceV1>(&wrong_count).is_err());

    let unknown_field = json.replacen('{', "{\"scientific_authority\":true,", 1);
    assert!(serde_json::from_str::<ValidationEvidenceV1>(&unknown_field).is_err());
}
