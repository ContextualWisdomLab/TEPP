use validation_core::WilsonCoverageEvidenceV1;

#[test]
fn inexact_count_zero_coverage_extreme_z_retains_non_unity_upper_bound() {
    let evidence = WilsonCoverageEvidenceV1 {
        sample_count: (1_u64 << 53) + 1,
        covered_count: 0,
        normal_critical_value: 1e16,
        empirical_coverage: 0.0,
        wilson_lower: 0.0,
        wilson_upper: f64::from_bits(0x3fef_ffff_ffff_ffff),
    };

    evidence
        .validate()
        .expect("exact inexact-count Wilson evidence should retain the represented miss mass");
    assert!(evidence.wilson_upper < 1.0);
}
