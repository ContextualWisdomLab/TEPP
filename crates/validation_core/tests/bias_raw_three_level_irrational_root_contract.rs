//! Public contract for exact raw three-level arithmetic with an irrational root.

use validation_core::bias_standard_error;

#[test]
fn exact_raw_three_level_refuses_nonrepresentable_square_root() {
    // Canonical translation of [0, 1, 3] is [-1, 0, 2]. Its raw products,
    // square sum, and radicand are all exact binary64 values:
    // 1 + 4 - (-2) = 7. The square root of 7 is irrational, so the bounded
    // exact three-level admission must refuse at the root representability
    // proof rather than treating the rounded hardware sqrt as authoritative.
    let truth = [0.0; 3];
    let residuals = [0.0, 1.0, 3.0];

    let standard_error = bias_standard_error(&truth, &residuals)
        .expect("irrational exact-root candidate falls back to represented geometry");

    // For [0, 1, 3], SE(mean)^2 = (1 + 9 - 3) / 9 = 7 / 9.
    assert_eq!(standard_error.to_bits(), 0x3fec_38aa_37c3_f68d);

    let permuted = [3.0, 0.0, 1.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves represented fallback geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
