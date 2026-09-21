//! Public contract for three-level fallback when the exact raw radicand overflows.

use validation_core::bias_standard_error;

#[test]
fn collapsed_subtraction_roundoffs_preserve_three_level_radicand_overflow() {
    let dominant_residual = f64::MAX;
    let offset = f64::from_bits(0x5fe4_0000_0000_0000); // 5 * 2^509
    let truth = [0.0, offset, -offset];
    let recovered = [dominant_residual; 3];

    // Error-free subtraction retains low terms [0, -offset, +offset] even though
    // all represented residuals round to the same finite `f64::MAX`. For these
    // offsets the individual squares, cross product and square sum are exact and
    // finite, but `x² + y² - xy = 3 * offset²` exceeds binary64 range. Exact
    // three-level admission must therefore refuse the raw radicand and let the
    // power-of-two-scaled translated path recover the finite represented SE.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("radicand overflow must fall back without rejecting dispersion");
    assert_eq!(standard_error.to_bits(), 0x5fd7_1811_16f4_3fe3);

    let permuted_truth = [-offset, 0.0, offset];
    let permuted_standard_error = bias_standard_error(&permuted_truth, &recovered)
        .expect("permutation preserves the represented low-term geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
