use validation_core::wilson_coverage_interval;

#[test]
fn exact_count_all_covered_does_not_overcorrect_below_final_ulp_resolution() {
    // n=3 and this finite binary64 z produce z^2 = 0x1.0000000000001p+985.
    // The exact represented-input endpoint 3 / (3 + z^2) rounds to the same
    // binary64 value as 3 / z^2 because the finite-count correction is below
    // the final quotient's half-ULP. Residual compensation must therefore keep
    // the already-correct direct quotient instead of forcing a one-ULP step.
    let z = f64::from_bits(0x5eb6_a09e_667f_3bcd);
    assert_eq!((z * z).to_bits(), 0x7d80_0000_0000_0001);

    let truth = [0.0; 3];
    let lower_bounds = [-1.0; 3];
    let upper_bounds = [1.0; 3];
    let (lower, upper) = wilson_coverage_interval(&truth, &lower_bounds, &upper_bounds, z)
        .expect("finite positive z with three covered intervals must produce Wilson evidence");

    assert_eq!(lower.to_bits(), 0x0277_ffff_ffff_ffff);
    assert_eq!(upper, 1.0);
}
