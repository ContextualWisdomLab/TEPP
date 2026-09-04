use validation_core::WilsonCoverageEvidenceV1;

#[test]
fn inexact_durable_count_preserves_positive_all_covered_lower_at_extreme_z() {
    // `sample_count = 2^53 + 1` is exact durable u64 provenance but is not
    // representable as binary64. With z = 1e20, the canonical all-covered
    // Wilson lower endpoint is n / (n + z^2) = 0x1.16c262777579dp-80.
    // Computing the complementary miss mass first rounds it to 1.0 and then
    // `1.0 - miss_mass` erases this ordinary representable positive endpoint.
    let json = r#"{
        "schema":"tepp.wilson_coverage_evidence.v1",
        "sample_count":9007199254740993,
        "covered_count":9007199254740993,
        "critical_value_kind":"standard_normal_z",
        "interval_sidedness":"two_sided",
        "normal_critical_value":1e20,
        "empirical_coverage":1.0,
        "wilson_lower":9.007199254740993e-25,
        "wilson_upper":1.0
    }"#;

    let evidence: WilsonCoverageEvidenceV1 = serde_json::from_str(json)
        .expect("exact durable sample count must preserve representable Wilson uncertainty");

    assert_eq!(evidence.wilson_lower.to_bits(), 0x3af1_6c26_2777_579d);
    assert!(evidence.wilson_lower > 0.0);
    assert_eq!(evidence.wilson_upper, 1.0);
}
