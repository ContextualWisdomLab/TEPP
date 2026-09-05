use validation_core::WilsonCoverageEvidenceV1;

#[test]
fn durable_wilson_endpoint_does_not_pre_round_exact_sample_count() {
    // The durable carrier owns exact u64 counts. This denominator is not exactly
    // representable in binary64: `n as f64` rounds 9_007_199_254_740_1013 down
    // to 9_007_199_254_740_1012. The exact integer ratio k/n still rounds to the
    // ordinary finite coverage value below, but pre-rounding n inside the Wilson
    // formula moves the lower endpoint one ULP upward.
    //
    // Evaluating the same Wilson score formula at the exact integer n and the
    // correctly rounded represented k/n gives lower = 0x1.2492482c43beap-3.
    let json = r#"{
        "schema":"tepp.wilson_coverage_evidence.v1",
        "sample_count":9007199254741013,
        "covered_count":1286742750677287,
        "critical_value_kind":"standard_normal_z",
        "interval_sidedness":"two_sided",
        "normal_critical_value":1.96,
        "empirical_coverage":0.1428571428571428,
        "wilson_lower":0.14285713563046382,
        "wilson_upper":0.14285715008382208
    }"#;

    let evidence: WilsonCoverageEvidenceV1 = serde_json::from_str(json)
        .expect("exact sample-count provenance must determine the Wilson endpoint");

    assert_eq!(evidence.empirical_coverage.to_bits(), 0x3fc2_4924_9249_2490);
    assert_eq!(evidence.wilson_lower.to_bits(), 0x3fc2_4924_82c4_3bea);
    assert_eq!(evidence.wilson_upper.to_bits(), 0x3fc2_4924_a1ce_0d41);
}

#[test]
fn durable_all_covered_lower_does_not_round_exact_sample_size_away() {
    // At n = 2^55 + 3 the exact all-covered Wilson lower endpoint for z = 1.96
    // rounds to next_down(1.0). Materializing n as f64 first, or adding z^2/n
    // to 1.0 before inversion, rounds the uncertainty away and returns exact 1.
    // The complement form 1 - (z^2/n)/(1 + z^2/n) preserves the representable
    // nonzero miss mass for this exact count provenance.
    let json = r#"{
        "schema":"tepp.wilson_coverage_evidence.v1",
        "sample_count":36028797018963971,
        "covered_count":36028797018963971,
        "critical_value_kind":"standard_normal_z",
        "interval_sidedness":"two_sided",
        "normal_critical_value":1.96,
        "empirical_coverage":1.0,
        "wilson_lower":0.9999999999999999,
        "wilson_upper":1.0
    }"#;

    let evidence: WilsonCoverageEvidenceV1 = serde_json::from_str(json)
        .expect("all-covered exact count provenance must retain Wilson uncertainty");

    assert_eq!(evidence.wilson_lower.to_bits(), 1.0_f64.to_bits() - 1);
    assert_eq!(evidence.wilson_upper, 1.0);
}
