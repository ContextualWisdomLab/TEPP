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

    let mut wrong_critical_value = evidence;
    wrong_critical_value.normal_critical_value = 2.576;
    assert_eq!(
        wrong_critical_value.validate(),
        Err(ValidationError::InvalidInput)
    );

    let mut wrong_endpoint = evidence;
    wrong_endpoint.wilson_upper = (wrong_endpoint.wilson_upper + 1.0) / 2.0;
    assert_eq!(wrong_endpoint.validate(), Err(ValidationError::InvalidInput));
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
}

#[test]
fn impossible_counts_and_unrepresentable_critical_value_fail_closed() {
    let evidence = canonical_evidence();

    let mut impossible_counts = evidence;
    impossible_counts.covered_count = impossible_counts.sample_count + 1;
    assert_eq!(impossible_counts.validate(), Err(ValidationError::InvalidInput));

    let mut invalid_z = evidence;
    invalid_z.normal_critical_value = 1e200;
    assert_eq!(invalid_z.validate(), Err(ValidationError::InvalidInput));
}
