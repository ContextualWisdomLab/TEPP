use validation_core::wilson_coverage_interval;

#[test]
fn exact_count_all_covered_preserves_partial_denominator_residual() {
    // z = 3 * 2^-28 is exactly representable. Squaring yields 9 * 2^-56,
    // while 1 + z^2 rounds upward to 1 + 2^-52. Dividing by that rounded
    // denominator lands two ULPs below one, even though the correctly rounded
    // represented-input Wilson endpoint 1 / (1 + 9 * 2^-56) is next_down(1).
    // The exact sample-count contribution is not fully absorbed, so the earlier
    // complete-absorption repair cannot recover this ordinary inexact sum.
    let z = f64::from_bits(0x3e48_0000_0000_0000);
    assert_eq!((z * z).to_bits(), 0x3ca2_0000_0000_0000);
    assert_eq!((1.0 + z * z).to_bits(), 0x3ff0_0000_0000_0001);

    let truth = [0.0];
    let lower_bounds = [-1.0];
    let upper_bounds = [1.0];
    let (lower, upper) = wilson_coverage_interval(&truth, &lower_bounds, &upper_bounds, z)
        .expect("one covered interval with finite positive z must produce Wilson evidence");

    assert_eq!(lower.to_bits(), 0x3fef_ffff_ffff_ffff);
    assert_eq!(upper, 1.0);
}

#[test]
fn exact_count_partial_denominator_compensation_keeps_correct_direct_rounding() {
    // z = 3 * 2^-27 also makes 1 + z^2 inexact, but the direct quotient is
    // already the correctly rounded represented-input endpoint. Compensation
    // must preserve that value rather than mechanically moving every inexact sum.
    let z = f64::from_bits(0x3e58_0000_0000_0000);
    assert_eq!((z * z).to_bits(), 0x3cc2_0000_0000_0000);
    assert_eq!((1.0 + z * z).to_bits(), 0x3ff0_0000_0000_0002);

    let truth = [0.0];
    let lower_bounds = [-1.0];
    let upper_bounds = [1.0];
    let (lower, upper) = wilson_coverage_interval(&truth, &lower_bounds, &upper_bounds, z)
        .expect("one covered interval with finite positive z must produce Wilson evidence");

    assert_eq!(lower.to_bits(), 0x3fef_ffff_ffff_fffc);
    assert_eq!(upper, 1.0);
}
