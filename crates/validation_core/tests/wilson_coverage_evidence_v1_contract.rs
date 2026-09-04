use validation_core::{ValidationError, WilsonCoverageEvidenceV1};

fn canonical_evidence() -> WilsonCoverageEvidenceV1 {
    let truth = [0.0, 1.0, 2.0, 3.0];
    let lower = [-0.5, 0.5, 1.5, 4.0];
    let upper = [0.5, 1.5, 2.5, 5.0];
    WilsonCoverageEvidenceV1::from_intervals(&truth, &lower, &upper, 1.96)
        .expect("canonical Wilson coverage evidence")
}

#[test]
fn versioned_wilson_coverage_evidence_round_trips_denominator_and_critical_value() {
    let evidence = canonical_evidence();
    assert_eq!(evidence.sample_count, 4);
    assert_eq!(evidence.covered_count, 3);
    assert_eq!(evidence.empirical_coverage, 0.75);
    assert_eq!(evidence.normal_critical_value, 1.96);
    assert!(evidence.wilson_lower <= evidence.empirical_coverage);
    assert!(evidence.empirical_coverage <= evidence.wilson_upper);

    let json = evidence.to_json().expect("canonical json");
    assert!(json.contains("\"schema\":\"tepp.wilson_coverage_evidence.v1\""));
    assert!(json.contains("\"critical_value_kind\":\"standard_normal_z\""));

    let decoded: WilsonCoverageEvidenceV1 = serde_json::from_str(&json).expect("decode");
    assert_eq!(decoded, evidence);
}

#[test]
fn tampered_denominator_critical_value_or_endpoint_fails_closed() {
    let evidence = canonical_evidence();

    let mut wrong_denominator = evidence;
    wrong_denominator.sample_count = 5;
    assert_eq!(
        wrong_denominator.validate(),
        Err(ValidationError::InvalidInput)
    );
    assert!(wrong_denominator.to_json().is_err());
    assert!(serde_json::to_string(&wrong_denominator).is_err());

    let mut wrong_critical_value = evidence;
    wrong_critical_value.normal_critical_value = 2.576;
    assert_eq!(
        wrong_critical_value.validate(),
        Err(ValidationError::InvalidInput)
    );

    let mut wrong_lower = evidence;
    wrong_lower.wilson_lower /= 2.0;
    assert_eq!(wrong_lower.validate(), Err(ValidationError::InvalidInput));

    let mut wrong_upper = evidence;
    wrong_upper.wilson_upper = (wrong_upper.wilson_upper + 1.0) / 2.0;
    assert_eq!(wrong_upper.validate(), Err(ValidationError::InvalidInput));

    let mut wrong_coverage = evidence;
    wrong_coverage.empirical_coverage = 0.5;
    assert_eq!(wrong_coverage.validate(), Err(ValidationError::InvalidInput));
}

#[test]
fn serde_requires_the_versioned_schema_and_standard_normal_critical_value_semantics() {
    let json = canonical_evidence().to_json().expect("canonical json");

    let wrong_schema = json.replace(
        "tepp.wilson_coverage_evidence.v1",
        "tepp.wilson_coverage_evidence.v2",
    );
    assert!(serde_json::from_str::<WilsonCoverageEvidenceV1>(&wrong_schema).is_err());

    let wrong_kind = json.replace("standard_normal_z", "student_t");
    assert!(serde_json::from_str::<WilsonCoverageEvidenceV1>(&wrong_kind).is_err());

    let unknown_field = json.replacen('{', "{\"confidence_level\":0.95,", 1);
    assert!(serde_json::from_str::<WilsonCoverageEvidenceV1>(&unknown_field).is_err());
}

#[test]
fn impossible_counts_and_numeric_domains_fail_closed() {
    let evidence = canonical_evidence();

    let mut zero_count = evidence;
    zero_count.sample_count = 0;
    assert_eq!(zero_count.validate(), Err(ValidationError::InvalidInput));

    let mut impossible_counts = evidence;
    impossible_counts.covered_count = impossible_counts.sample_count + 1;
    assert_eq!(impossible_counts.validate(), Err(ValidationError::InvalidInput));

    for invalid_z in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e200] {
        let mut invalid = evidence;
        invalid.normal_critical_value = invalid_z;
        assert_eq!(invalid.validate(), Err(ValidationError::InvalidInput));
    }

    for invalid_probability in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
        let mut invalid = evidence;
        invalid.empirical_coverage = invalid_probability;
        assert_eq!(invalid.validate(), Err(ValidationError::InvalidInput));
    }
}

#[test]
fn constructor_preserves_existing_input_and_configuration_error_contracts() {
    assert_eq!(
        WilsonCoverageEvidenceV1::from_intervals(&[], &[], &[], 1.96),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        WilsonCoverageEvidenceV1::from_intervals(&[0.0], &[1.0], &[0.0], 1.96),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        WilsonCoverageEvidenceV1::from_intervals(&[0.0], &[-1.0], &[1.0], 0.0),
        Err(ValidationError::InvalidConfiguration)
    );
    assert_eq!(
        WilsonCoverageEvidenceV1::from_intervals(&[0.0], &[-1.0], &[1.0], 1e200),
        Err(ValidationError::InvalidConfiguration)
    );
}
