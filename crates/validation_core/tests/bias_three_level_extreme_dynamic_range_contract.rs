//! Public contract for three-level normalization refusal at extreme residual scale.

use validation_core::bias_standard_error;

#[test]
fn three_level_extreme_dynamic_range_preserves_finite_standard_error() {
    let huge = f64::from_bits(0x7fe0_0000_0000_0000); // 2^1023
    let tiny = f64::from_bits(0x0000_0000_0000_0001); // 2^-1074
    let truth = [0.0; 3];
    let recovered = [0.0, tiny, huge];

    // Canonical translation around zero preserves all three represented values.
    // The direct three-level proof must retry after raw square overflow, but the
    // exact 2^1023 normalization would erase the nonzero minimum subnormal. That
    // proof therefore refuses instead of collapsing a represented level to zero.
    // The fallback remains finite: for [0, b, A],
    // SE(mean)^2 = (A^2 + b^2 - Ab) / 9, whose correctly rounded binary64 value
    // for A=2^1023 and b=2^-1074 is A/3 = 0x7fc5555555555555.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("three-level extreme dynamic range retains a finite standard error");
    assert_eq!(standard_error.to_bits(), 0x7fc5_5555_5555_5555);

    let permuted_recovered = [huge, 0.0, tiny];
    let permuted_standard_error = bias_standard_error(&truth, &permuted_recovered)
        .expect("permutation preserves the represented three-level geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
