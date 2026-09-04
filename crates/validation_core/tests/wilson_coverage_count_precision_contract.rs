use validation_core::WilsonCoverageEvidenceV1;

#[test]
fn durable_counts_do_not_collapse_one_uncovered_case_to_all_covered() {
    // n = 2^53 + 1 and k = 2^53 are distinct integer counts, but converting each
    // count independently to binary64 rounds both to 2^53. The durable count
    // provenance must therefore form the represented proportion from the small
    // uncovered complement rather than erase the one observed miss.
    let json = r#"{
        "schema":"tepp.wilson_coverage_evidence.v1",
        "sample_count":9007199254740993,
        "covered_count":9007199254740992,
        "critical_value_kind":"standard_normal_z",
        "interval_sidedness":"two_sided",
        "normal_critical_value":1.96,
        "empirical_coverage":0.9999999999999999,
        "wilson_lower":0.9999999999999993,
        "wilson_upper":1.0
    }"#;

    let evidence: WilsonCoverageEvidenceV1 =
        serde_json::from_str(json).expect("large-count Wilson evidence must remain reproducible");

    assert_eq!(evidence.sample_count as u64, 9_007_199_254_740_993);
    assert_eq!(evidence.covered_count as u64, 9_007_199_254_740_992);
    assert_eq!(evidence.empirical_coverage.to_bits(), 0x3fef_ffff_ffff_ffff);
    assert_eq!(evidence.wilson_lower.to_bits(), 0x3fef_ffff_ffff_fffa);
    assert_eq!(evidence.wilson_upper, 1.0);

    let round_trip = evidence.to_json().expect("validated durable evidence");
    let decoded: WilsonCoverageEvidenceV1 =
        serde_json::from_str(&round_trip).expect("round-trip durable evidence");
    assert_eq!(decoded, evidence);
}
