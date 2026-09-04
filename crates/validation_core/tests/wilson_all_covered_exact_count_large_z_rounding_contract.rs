use validation_core::wilson_coverage_interval;

#[test]
fn exact_count_all_covered_preserves_large_z_denominator_residual() {
    // This z is the binary64 value 0x1.fffffffffffffp+29. Squaring it yields
    // 0x1.ffffffffffffep+59. For n=3, adding the exact sample count to z^2
    // rounds back to z^2, but the exact represented-input Wilson endpoint
    // 3 / (3 + z^2) rounds one ULP below the quotient formed from that rounded
    // denominator. The finite sample-count contribution must not disappear.
    let z = f64::from_bits(0x41cf_ffff_ffff_ffff);
    assert_eq!((z * z).to_bits(), 0x43af_ffff_ffff_fffe);

    let truth = [0.0; 3];
    let lower_bounds = [-1.0; 3];
    let upper_bounds = [1.0; 3];
    let (lower, upper) = wilson_coverage_interval(&truth, &lower_bounds, &upper_bounds, z)
        .expect("three covered intervals with finite positive z must produce Wilson evidence");

    assert_eq!(lower.to_bits(), 0x3c48_0000_0000_0001);
    assert_eq!(upper, 1.0);
}

#[test]
fn exact_count_all_covered_keeps_correct_power_of_two_large_z_rounding() {
    // At z=2^30 the sample count is likewise fully absorbed by z^2 when the
    // denominator is formed, but the ordinary quotient already lands on the
    // correctly rounded represented-input endpoint. Residual compensation must
    // preserve that value rather than forcing a one-ULP adjustment.
    let z = f64::from_bits(0x41d0_0000_0000_0000);
    assert_eq!((z * z).to_bits(), 0x43b0_0000_0000_0000);

    let truth = [0.0; 3];
    let lower_bounds = [-1.0; 3];
    let upper_bounds = [1.0; 3];
    let (lower, upper) = wilson_coverage_interval(&truth, &lower_bounds, &upper_bounds, z)
        .expect("large finite power-of-two z must remain valid Wilson evidence");

    assert_eq!(lower.to_bits(), 0x3c48_0000_0000_0000);
    assert_eq!(upper, 1.0);
}
