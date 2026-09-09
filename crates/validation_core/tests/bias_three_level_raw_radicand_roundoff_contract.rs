//! Public contract for raw three-level radicand-roundoff fallback.

use validation_core::bias_standard_error;

#[test]
fn three_level_refuses_inexact_raw_radicand_subtraction() {
    let dominant = 2.0_f64.powi(26);
    let minor = dominant - 1.0;
    let truth = [0.0; 3];

    // The canonical exact translation of `[0, A, -B]` keeps zero as the
    // minimum-range anchor for `A = 2^26` and `B = 2^26 - 1`. Every raw square
    // and the cross-product is exactly representable, and `A^2 + B^2` is still
    // an exact integer below 2^53. The exact radicand
    // `A^2 + B^2 + A*B = 13_510_798_680_784_897` is odd and exceeds 2^53, so
    // the final subtraction/addition cannot be represented exactly. Exact
    // three-level admission must therefore refuse at the raw radicand roundoff
    // proof and let the represented translated path finish the estimate.
    let dominant_first = [0.0, dominant, -minor];
    let first = bias_standard_error(&truth, &dominant_first)
        .expect("raw radicand roundoff falls back to represented sample");
    assert_eq!(first.to_bits(), 0x4182_79a7_4340_fe34);

    // Observation order must not alter the translated geometry or fallback.
    let minor_first = [0.0, -minor, dominant];
    let second = bias_standard_error(&truth, &minor_first)
        .expect("permuted raw radicand fallback preserves represented sample");
    assert_eq!(second.to_bits(), first.to_bits());
}
