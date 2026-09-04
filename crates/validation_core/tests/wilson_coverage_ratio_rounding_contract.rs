use validation_core::WilsonCoverageEvidenceV1;

#[test]
fn durable_coverage_ratio_is_correctly_rounded_before_wilson_projection() {
    // These integer counts are exact provenance. Their quotient lies one binary64
    // ULP below the result obtained by independently rounding both integers to
    // f64 before division:
    //   k/n = 4_503_599_627_370_396 / 9_007_199_254_740_993
    //       -> 0x1.fffffffffff37p-2 (round-to-nearest, ties-to-even)
    // while `(k as f64) / (n as f64)` becomes 0x1.fffffffffff38p-2.
    let json = r#"{
        "schema":"tepp.wilson_coverage_evidence.v1",
        "sample_count":9007199254740993,
        "covered_count":4503599627370396,
        "critical_value_kind":"standard_normal_z",
        "interval_sidedness":"two_sided",
        "normal_critical_value":1.96,
        "empirical_coverage":0.49999999999998884,
        "wilson_lower":0.49999998967401105,
        "wilson_upper":0.5000000103259667
    }"#;

    let evidence: WilsonCoverageEvidenceV1 = serde_json::from_str(json)
        .expect("exact count provenance must determine the correctly rounded coverage ratio");

    assert_eq!(evidence.empirical_coverage.to_bits(), 0x3fdf_ffff_ffff_ff37);
    assert_eq!(evidence.wilson_lower.to_bits(), 0x3fdf_ffff_f4e9_9d20);
    assert_eq!(evidence.wilson_upper.to_bits(), 0x3fe0_0000_058b_30a8);

    let round_trip = evidence.to_json().expect("validated durable evidence");
    let decoded: WilsonCoverageEvidenceV1 =
        serde_json::from_str(&round_trip).expect("round-trip durable evidence");
    assert_eq!(decoded, evidence);
}
