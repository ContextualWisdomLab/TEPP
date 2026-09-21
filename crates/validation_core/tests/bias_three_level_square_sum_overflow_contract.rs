//! Public contract for three-level fallback when the raw square sum overflows.

use validation_core::bias_standard_error;

#[test]
fn collapsed_subtraction_roundoffs_preserve_three_level_square_sum_overflow() {
    let dominant_residual = f64::MAX;
    let offset = f64::from_bits(0x5fe8_0000_0000_0000); // 1.5 * 2^511
    let truth = [0.0, offset, -offset];
    let recovered = [dominant_residual; 3];

    // Each represented subtraction rounds to the same finite `f64::MAX`, while
    // error-free subtraction retains low terms [0, -offset, +offset]. The bounded
    // exact pair-distance route therefore refuses rounded input subtraction and
    // the bias fallback evaluates the translation-invariant low-term geometry.
    // Each offset square and their cross product is finite and exact, but the two
    // squares sum past binary64 range. The direct three-level proof must refuse
    // that intermediate without rejecting the represented dispersion; the
    // power-of-two-scaled translated path then returns offset / sqrt(3).
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("square-sum overflow must fall back without losing dispersion");
    assert_eq!(standard_error.to_bits(), 0x5fdb_b67a_e858_4caa);

    let permuted_truth = [-offset, 0.0, offset];
    let permuted_standard_error = bias_standard_error(&permuted_truth, &recovered)
        .expect("permutation preserves the represented low-term geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
