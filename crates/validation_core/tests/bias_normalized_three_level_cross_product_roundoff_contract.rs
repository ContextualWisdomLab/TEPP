//! Public contract for normalized three-level cross-product-roundoff fallback.

use validation_core::bias_standard_error;

#[test]
fn normalized_three_level_refuses_inexact_cross_product_after_square_underflow() {
    let dominant = 1.5_f64 * 2.0_f64.powi(600);
    let minor = f64::from_bits(1.0_f64.to_bits() + 1);
    let truth = [0.0; 3];

    // The n=3 neutral-zero proof cannot fit the 651-bit coefficient alignment,
    // and pairwise subtraction of the dominant and minor residuals is inexact.
    // The bias fallback therefore reaches the three-level identity. The dominant
    // square overflows before normalization; afterwards the offsets are
    // `1.5` and `(1 + 2^-52) * 2^-600`. The minor square and its fused residual
    // both underflow to zero, while the normalized cross-product remains normal
    // but is not exactly representable. Exact admission must refuse at the
    // cross-product FMA proof rather than treating that rounded product as exact.
    // For the represented sample `[0, A, b]`,
    // `SE(mean)^2 = (A^2 + b^2 - A*b) / 9`, whose correctly rounded binary64
    // result at this separation is exactly `2^599`.
    let dominant_first = [0.0, dominant, minor];
    let first = bias_standard_error(&truth, &dominant_first)
        .expect("inexact normalized cross-product falls back to represented sample");
    assert_eq!(first.to_bits(), 2.0_f64.powi(599).to_bits());

    // Permuting the two nonzero residuals must preserve the same represented
    // geometry and exercise the symmetric normalized product proof.
    let minor_first = [0.0, minor, dominant];
    let second = bias_standard_error(&truth, &minor_first)
        .expect("permuted cross-product fallback preserves represented sample");
    assert_eq!(second.to_bits(), first.to_bits());
}
