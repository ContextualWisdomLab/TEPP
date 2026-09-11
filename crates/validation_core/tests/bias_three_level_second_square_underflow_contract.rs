//! Public contract for three-level proof fallback when only the minor square underflows.

use validation_core::bias_standard_error;

#[test]
fn minor_level_square_underflow_preserves_represented_three_level_dispersion() {
    let minor = f64::from_bits(0x1a70_0000_0000_0000); // 2^-600
    let truth = [0.0; 3];
    let recovered = [0.0, 1.0, minor];

    // The bounded represented-input route refuses the 600-bit coefficient span.
    // Canonical exact translation then reaches the three-level proof as [0, 1,
    // 2^-600]. The major square and cross-product are finite and nonzero, while
    // only the minor square underflows to zero. That is a real proof-refusal
    // outcome, not an invalid input: the general translated path must retain the
    // finite represented sample and return its correctly rounded SE.
    //
    // SE(mean)^2 = (1 + 2^-1200 - 2^-600) / 9, whose correctly rounded binary64
    // square root is the same representation as 1/3: 0x3fd5555555555555.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("minor-square underflow must fall back without losing the sample");
    assert_eq!(standard_error.to_bits(), 0x3fd5_5555_5555_5555);

    let permuted_recovered = [minor, 0.0, 1.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted_recovered)
        .expect("permutation preserves the represented three-level geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
