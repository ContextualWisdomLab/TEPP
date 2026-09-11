//! Public contract for raw three-level radicand-roundoff fallback.

use validation_core::bias_standard_error;

#[test]
fn three_level_refuses_inexact_raw_radicand_subtraction() {
    let unit = 2.0_f64.powi(-80);
    let dominant = 2.0_f64.powi(26) * unit;
    let minor = (2.0_f64.powi(26) - 1.0) * unit;
    let recovered = [1.0; 3];

    // Public subtraction first rounds every residual high part to `1.0`, while
    // its error-free low terms retain `[0, +A, -B]` for
    // `A = 2^26 * 2^-80` and `B = (2^26 - 1) * 2^-80`. That nonzero
    // subtraction roundoff makes the bounded bias_se route refuse and sends the
    // represented low-term geometry through the production bias fallback.
    //
    // In that fallback, every raw square and the cross-product is exactly
    // representable, and `A^2 + B^2` is still exact. The remaining radicand has
    // integer coefficient
    // `2^52 + (2^26 - 1)^2 + 2^26(2^26 - 1)
    //  = 13_510_798_680_784_897`,
    // which is odd and above 2^53. Exact three-level admission must therefore
    // refuse at the raw radicand roundoff proof instead of accepting its rounded
    // binary64 value. The represented translated path then returns the correctly
    // rounded `sqrt(A^2 + B^2 + A*B) / 3` result below.
    let truth = [0.0, -dominant, minor];
    let first = bias_standard_error(&truth, &recovered)
        .expect("raw radicand roundoff falls back to represented sample");
    assert_eq!(first.to_bits(), 0x3c82_79a7_4340_fe34);

    // Observation order must not alter either the canonical low-term geometry
    // or the represented fallback result.
    let permuted_truth = [0.0, minor, -dominant];
    let second = bias_standard_error(&permuted_truth, &recovered)
        .expect("permuted raw radicand fallback preserves represented sample");
    assert_eq!(second.to_bits(), first.to_bits());
}
