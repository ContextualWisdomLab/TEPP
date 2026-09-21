//! Public contract for correctly rounded three-level SE at extreme residual scale.

use validation_core::bias_standard_error;

#[test]
fn three_level_extreme_dynamic_range_returns_exact_finite_result() {
    let huge = f64::from_bits(0x7fe0_0000_0000_0000); // 2^1023
    let tiny = f64::from_bits(0x0000_0000_0000_0001); // 2^-1074
    let truth = [0.0; 3];
    let recovered = [0.0, tiny, huge];

    // Canonical translation preserves all three represented values. Although
    // exact 2^1023 normalization erases the minimum-subnormal offset, its exact
    // contribution is far below the nearest binary64 midpoint around A/3. The
    // represented three-level target therefore rounds to A/3 without discarding
    // scientific geometry or falling back to the one-ULP-high mean/deviation path.
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(standard_error.to_bits(), 0x7fc5_5555_5555_5555);

    let permuted_recovered = [huge, 0.0, tiny];
    let permuted_standard_error =
        bias_standard_error(&truth, &permuted_recovered).expect("permuted representable SE");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
