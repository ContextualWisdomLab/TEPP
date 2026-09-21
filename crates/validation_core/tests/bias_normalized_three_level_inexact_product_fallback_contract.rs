//! Public contract for normalized three-level product-inexact fallback.

use validation_core::bias_standard_error;

#[test]
fn normalized_three_level_refuses_inexact_square_without_losing_dynamic_range() {
    let dominant = 2.0_f64.powi(600);
    let minor = 1.1_f64 * 2.0_f64.powi(520);
    let truth = [0.0; 3];

    // The bounded n=3 admission cannot express the 131-bit coefficient shift
    // between these exact represented residuals, and the pairwise reference
    // rejects the rounded dominant-minus-minor distance. The fallback therefore
    // reaches the exact three-level identity. Raw squares overflow, while exact
    // power-of-two normalization yields [1, 1.1*2^-80]. The minor square is not
    // exactly representable, so that normalized exact-admission path must refuse
    // at its FMA proof rather than treating the rounded product as authoritative.
    // The general translated path still preserves the represented sample. For
    // [0, A, b], SE(mean)^2 = (A^2 + b^2 - A*b) / 9; at this separation the
    // correctly rounded binary64 result is A/3 = 0x6555555555555555.
    let dominant_first = [0.0, dominant, minor];
    let first = bias_standard_error(&truth, &dominant_first)
        .expect("inexact normalized square falls back to the represented sample");
    assert_eq!(first.to_bits(), 0x6555_5555_5555_5555);

    // Swapping the two nonzero residuals exercises the symmetric first-product
    // proof and must not change the scientific result.
    let minor_first = [0.0, minor, dominant];
    let second = bias_standard_error(&truth, &minor_first)
        .expect("permuted inexact normalized square preserves the same geometry");
    assert_eq!(second.to_bits(), first.to_bits());
}
